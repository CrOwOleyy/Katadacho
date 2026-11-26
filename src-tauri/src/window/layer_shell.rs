//! L'Island sous Wayland. Une fenêtre ordinaire ne peut ni se placer ni rester au-dessus : le
//! compositeur la range où il veut, et les gestionnaires en tuiles (niri, Hyprland, Sway) l'ajoutent
//! même à la mise en page, cadre de focus compris. Le protocole wlr-layer-shell, fait pour les barres
//! et les panneaux (KDE, niri, Hyprland, Sway), règle tout : l'Island devient une surface ancrée en
//! haut de l'écran, au-dessus des fenêtres et hors de la mise en page.
//!
//! GTK n'accepte layer-shell que sur une fenêtre jamais affichée. Celle de Tauri l'a déjà été : son
//! contenu (la webview) est donc déplacé dans une nouvelle fenêtre GTK, et l'ancienne reste cachée.

use std::cell::RefCell;

use gtk::cairo::{Operator, RectangleInt, Region};
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tauri::{AppHandle, WebviewWindow};

use super::click_through::IslandBounds;

/// Marge autour de la pilule, pour que le survol ne clignote pas au bord.
const INPUT_MARGIN_PX: f64 = 8.0;

thread_local! {
    /// Fenêtre layer-shell. GTK n'est utilisable que sur son thread principal : elle n'en sort pas.
    static LAYER_WINDOW: RefCell<Option<gtk::ApplicationWindow>> = const { RefCell::new(None) };
}

/// Transforme l'Island en surface layer-shell. Retourne faux quand le compositeur ne le permet pas
/// (GNOME, session X11) : la fenêtre ordinaire est alors gardée. Doit tourner sur le thread principal.
pub fn install(window: &WebviewWindow) -> bool {
    // Échappatoire si un compositeur gère mal layer-shell : la fenêtre ordinaire est gardée.
    if std::env::var_os("KATADACHO_NO_LAYER_SHELL").is_some() || !gtk_layer_shell::is_supported() {
        return false;
    }
    let (Ok(original), Ok(content)) = (window.gtk_window(), window.default_vbox()) else {
        return false;
    };
    let Some(application) = original.application() else {
        return false;
    };
    let (width, height) = original.size();

    let _ = window.hide();
    let layer = gtk::ApplicationWindow::new(&application);
    // Sans visuel RGBA, la zone transparente autour de la pilule serait peinte en noir.
    if let Some(visual) = GtkWindowExt::screen(&layer).and_then(|screen| screen.rgba_visual()) {
        layer.set_visual(Some(&visual));
    }
    layer.set_app_paintable(true);
    // Efface la fenêtre avant chaque dessin, comme le fait Tao pour ses fenêtres transparentes :
    // sinon chaque étape d'une animation reste peinte et laisse une traînée.
    layer.connect_draw(|_, context| {
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.set_operator(Operator::Source);
        let _ = context.paint();
        context.set_operator(Operator::Over);
        Propagation::Proceed
    });
    layer.set_decorated(false);
    original.remove(&content);
    layer.add(&content);
    layer.set_size_request(width, height);

    layer.init_layer_shell();
    layer.set_namespace("katadacho");
    layer.set_layer(Layer::Overlay);
    // Ancrée en haut seulement : le compositeur la centre horizontalement.
    layer.set_anchor(Edge::Top, true);
    // Ne réserve aucune place et se pose sous une barre existante (Waybar, panneau KDE).
    layer.set_exclusive_zone(0);
    // Le clavier n'est pris qu'au clic, pour le lanceur et les réglages.
    layer.set_keyboard_mode(KeyboardMode::OnDemand);
    // Rien n'est cliquable tant que le frontend n'a pas donné la place de la pilule.
    layer.input_shape_combine_region(Some(&Region::create()));
    layer.show_all();

    LAYER_WINDOW.with(|cell| *cell.borrow_mut() = Some(layer));
    true
}

/// Rectangle cliquable, marge comprise, en pixels de la fenêtre.
fn input_rectangle(bounds: IslandBounds) -> RectangleInt {
    let left = (bounds.x - INPUT_MARGIN_PX).floor().max(0.0);
    let top = (bounds.y - INPUT_MARGIN_PX).floor().max(0.0);
    let right = (bounds.x + bounds.width + INPUT_MARGIN_PX).ceil();
    let bottom = (bounds.y + bounds.height + INPUT_MARGIN_PX).ceil();
    RectangleInt::new(left as i32, top as i32, (right - left).max(0.0) as i32, (bottom - top).max(0.0) as i32)
}

/// Seule la pilule reçoit la souris : ailleurs, les clics traversent vers les fenêtres du dessous.
/// Sans effet quand l'Island n'est pas en layer-shell.
pub fn set_input_region(app: &AppHandle, bounds: IslandBounds) {
    let _ = app.run_on_main_thread(move || {
        LAYER_WINDOW.with(|cell| {
            if let Some(layer) = cell.borrow().as_ref() {
                layer.input_shape_combine_region(Some(&Region::create_rectangle(&input_rectangle(bounds))));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_input_rectangle_adds_the_margin_and_stays_inside_the_window() {
        let rectangle = input_rectangle(IslandBounds { x: 300.5, y: 2.0, width: 200.0, height: 40.0 });
        assert_eq!((rectangle.x(), rectangle.y()), (292, 0));
        assert_eq!((rectangle.width(), rectangle.height()), (217, 50));
    }
}
