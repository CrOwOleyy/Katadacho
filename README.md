# Katadacho

Une « Dynamic Island » pour Windows 11, collée en haut de l'écran, qui regroupe les réglages système, la télémétrie du PC, un presse-papier persistant, un lanceur d'applications, un minuteur et les notifications Windows. Application de bureau en Tauri v2 (Rust) avec une interface React.

> Statut : version 0.1.0, en développement actif. Seul Windows 11 est géré pour l'instant.

## Fonctionnalités

### Déjà disponibles

| Domaine | Ce que ça fait |
|---|---|
| **Island** | Pilule transparente de 180 px en haut de l'écran. Elle s'ouvre après un court survol (ou au clic), affiche l'heure, la pochette et une onde sonore du média en cours, et se cache d'elle-même. Les zones transparentes laissent passer les clics. |
| **Controls** | Volume et changement de périphérique de sortie en un clic, luminosité (écrans de portable et écrans externes DDC/CI), Wi-Fi et Bluetooth (activation, réseau connecté, appareils connectés), et contrôles média (lecture, pause, suivant, précédent, déplacement dans le morceau). |
| **Dashboard** | CPU, RAM, GPU, disque et réseau en temps réel avec historique, Top 5 des processus regroupés par application avec bouton pour les terminer, et speedtest intégré (latence, gigue, débit descendant et montant). |
| **Clipboard** | Historique infini de tous les `Ctrl+C` (texte, liens, couleurs Hex, code, images), recherche instantanée, filtres, épinglage, suppression et collage direct dans la fenêtre précédente. |
| **Launcher** | Lanceur façon Everything : applications du menu Démarrer, applications du Microsoft Store, fichiers et dossiers de ton profil, recherche floue en mémoire, lancement en administrateur et ouverture de l'emplacement. |
| **Minuteur** | Pomodoro (concentration, pause courte, pause longue) et minuteur simple. Une puce avec le temps restant s'affiche dans la pilule, avec un son à la fin de chaque phase. |
| **Notifications** | Les notifications Windows s'affichent en bannière dans la pilule, avec file d'attente, durée réglable et liste d'applications ignorées. |
| **Réglages** | Vitesse et délais des animations, disparition automatique, durées du Pomodoro, options des notifications. Tout est enregistré. |

### Prévu, pas encore fait

- Gestionnaire de fonds d'écran façon Niri, avec palette de couleurs qui colore l'Island.
- Les 4 Slots personnalisables.
- Pont IA sans clé API (serveur MCP pour Claude Desktop, UI Automation, WebSocket local, extension navigateur).
- « Circle to Ask » : entourer une zone de l'écran pour la soumettre à une IA.
- Routines et actions planifiées.
- Pont vers le service Everything de voidtools, pour étendre la recherche à tout le disque.

## Prérequis

| Outil | Version | Remarque |
|---|---|---|
| Windows | 11 | Testé sur Windows 11. Windows 10 non testé. |
| Visual Studio Build Tools | 2022 | Charge de travail « Développement Desktop en C++ » (MSVC et SDK Windows). |
| Rust | 1.85 ou plus | Le projet utilise l'édition 2024. Développé avec 1.99. |
| Node.js | 20.19 ou plus (ou 22.12 ou plus) | Exigé par Vite 8. Développé avec 24. |
| pnpm | Récent | Développé avec 12. Aucune version minimale n'est imposée par le projet. |
| WebView2 | Dernière | Déjà présent sur Windows 11. |

### Smart App Control

Smart App Control bloque les exécutables non signés, dont celui que Cargo compile. Dans ce cas `pnpm tauri dev` échoue avec :

```text
Une stratégie de contrôle d'application a bloqué ce fichier. (os error 4551)
```

Pour développer, désactive-le dans **Sécurité Windows**, **Contrôle des applications et du navigateur**, **Paramètres de Contrôle intelligent des applications**. Attention : c'est irréversible sans réinstaller Windows. Les autres protections (Defender, SmartScreen) restent actives.

## Installation et lancement

```bash
pnpm install
```

Lancer l'application en développement (la première compilation Rust prend plusieurs minutes) :

```bash
pnpm tauri dev
```

Vérifier le frontend seul (typage TypeScript strict puis build Vite) :

```bash
pnpm build
```

Lancer les tests Rust :

```bash
cd src-tauri && cargo test --lib
```

Un test, `lists_real_toasts`, est ignoré par défaut : il lit de vraies notifications Windows. Pour le lancer à la main, après avoir reçu une notification :

```bash
cd src-tauri && cargo test --lib lists_real_toasts -- --ignored --nocapture
```

## Utilisation

- **Ouvrir le menu** : survole la pilule environ 300 ms, ou clique dessus. Le délai se règle dans Réglages.
- **Fermer le menu** : quitte la pilule, appuie sur `Échap`, ou clique sur la croix.
- **Lanceur** : `Alt + Espace` depuis n'importe quelle application. Si ce raccourci est déjà pris (menu système, PowerToys Run), `Ctrl + Alt + Espace` est utilisé à la place.
  - `↑` `↓` pour naviguer, `Entrée` pour lancer, `Ctrl + Entrée` pour lancer en administrateur, `Tab` pour ouvrir l'emplacement.
- **Barre cachée** : quand l'Island se cache, une fine barre reste en haut de l'écran. La survoler la fait réapparaître.
- **Notifications** : au premier lancement, autorise l'accès dans Réglages, section Notifications Windows.

## Données et vie privée

Tout reste sur ta machine, dans `%LocalAppData%\Katadacho` :

| Élément | Emplacement |
|---|---|
| Base SQLite (historique du presse-papier, réglages) | `katadacho.db` |
| Images copiées et miniatures | `clipboard_images\` |
| Icônes mises en cache | `icons\` |

Points d'attention :

- **Le presse-papier enregistre tout ce que tu copies**, y compris un mot de passe copié depuis un gestionnaire. Il n'existe pas encore de filtre pour les exclure.
- **Les notifications peuvent afficher leur contenu** dans la pilule. Si tu partages ton écran, désactive « Afficher le contenu » dans Réglages.
- **Le speedtest contacte `speed.cloudflare.com`**, seul appel réseau de l'application.

## Architecture

```text
React (WebView2)  <--- commandes et événements Tauri --->  Rust (API Windows natives)
```

- Une seule fenêtre `island`, de 900 par 520 px, transparente, sans décorations et toujours au premier plan. La pilule n'en occupe qu'une partie.
- **Click-through** : le frontend envoie à Rust le rectangle exact de la pilule à chaque changement de taille. Un thread Rust suit le curseur et ne laisse la fenêtre capter la souris que lorsqu'elle est sur la pilule. Il tourne à environ 60 Hz près de la pilule et 10 Hz ailleurs.
- **Actions ponctuelles** : commandes `#[tauri::command]` asynchrones, qui travaillent sur des threads dédiés pour ne jamais bloquer l'interface.
- **Flux en temps réel** : événements Tauri (`system-stats`, `clipboard-updated`, `media-changed`, `audio-level`, `timer-update`, `notification-arrived`, `settings-changed`) et un `Channel` pour la progression du speedtest.
- **Zéro appel à PowerShell** : tout passe par la crate `windows` (Win32, COM et WinRT).
- Les collecteurs ne tournent que lorsqu'ils servent : la télémétrie quand le Dashboard est ouvert, le niveau sonore quand un média joue, le minuteur seulement pendant un décompte.

### Structure du projet

```text
src/                         Interface React
  island/IslandApp.tsx       Pilule, onglets, ouverture et fermeture
  features/                  controls, dashboard, clipboard, launcher, media,
                             timer, notifications, settings
  components/                Briques d'interface (cartes, curseurs, anneaux)
  hooks/                     Hooks Tauri (stats, média, minuteur, notifications)
  store/SettingsContext.tsx  Réglages partagés, sauvegarde différée
  lib/                       Appels IPC typés, types, presets de mouvement

src-tauri/src/               Backend Rust
  window/                    Positionnement et click-through
  controls/                  Audio (IPolicyConfig), luminosité, Wi-Fi, radios
  telemetry/                 CPU, RAM, disque, réseau, processus, GPU
  speedtest/                 Latence, téléchargement, envoi
  clipboard/                 Écoute Win32, lecture, stockage, collage
  launcher/                  Index, recherche floue, icônes, raccourci global
  media/                     Média en cours, contrôles, niveau sonore
  timer/                     Machine à états du Pomodoro et thread de décompte
  notifications/             Lecture des toasts Windows
  settings.rs, db/           Réglages et SQLite avec migrations
```

## Dépendances

### Frontend (npm)

| Paquet | Rôle |
|---|---|
| `react`, `react-dom` 19 | Interface |
| `motion` | Animations à ressort (import `motion/react`, ex-Framer Motion) |
| `lucide-react` | Icônes |
| `@tauri-apps/api` 2 | Appels de commandes, événements, canaux |
| `@fontsource-variable/inter`, `@fontsource-variable/jetbrains-mono` | Polices, embarquées |
| `zustand`, `react-markdown`, `remark-gfm` | Installés pour les prochaines tâches (Slots, réponses de l'IA). Pas encore utilisés. |
| `@tauri-apps/plugin-opener`, `plugin-autostart`, `plugin-global-shortcut` | Installés avec les plugins. Les plugins sont utilisés côté Rust, ces paquets JS ne le sont pas encore. |

Outils de développement : `vite` 8, `@vitejs/plugin-react`, `typescript` 6, `tailwindcss` 4 avec `@tailwindcss/vite`, `@tauri-apps/cli` 2, `@types/react` et `@types/react-dom`.

### Backend (crates Rust)

| Crate | Rôle |
|---|---|
| `tauri` 2 | Framework de l'application |
| `tauri-plugin-global-shortcut` | Raccourci global du lanceur |
| `tauri-plugin-single-instance` | Une seule instance (le rappel est vide pour l'instant) |
| `tauri-plugin-autostart` | Démarrage avec Windows (initialisé, pas encore configuré) |
| `tauri-plugin-opener` | Ouverture de liens et fichiers (initialisé) |
| `windows` 0.62 | API Win32, COM et WinRT, avec les features listées plus bas |
| `sysinfo` | CPU, RAM, disques, réseau, processus |
| `nvml-wrapper` | GPU NVIDIA (le repli AMD et Intel passe par DXGI et PDH) |
| `wmi` | Luminosité des écrans de portable |
| `rusqlite` (bundled) | SQLite embarqué : historique du presse-papier, réglages |
| `nucleo-matcher` | Recherche floue du lanceur |
| `reqwest` (rustls, stream, http2), `futures-util`, `bytes`, `tokio` | Speedtest en flux |
| `image` (PNG) | Images du presse-papier et icônes |
| `sha2` | Dédoublonnage du presse-papier et noms du cache d'icônes |
| `base64` | Envoi des images au frontend |
| `serde`, `serde_json` | Sérialisation des échanges et des réglages |

Features de la crate `windows`, par usage :

| Usage | Features |
|---|---|
| Fenêtre et clic traversant | `Win32_Foundation`, `Win32_UI_WindowsAndMessaging` |
| Audio | `Win32_Media_Audio`, `Win32_Media_Audio_Endpoints`, `Win32_Devices_FunctionDiscovery`, `Win32_UI_Shell_PropertiesSystem`, `Win32_System_Variant`, `Win32_System_Com_StructuredStorage` |
| Luminosité | `Win32_Devices_Display`, `Win32_Graphics_Gdi` |
| Wi-Fi | `Win32_NetworkManagement_WiFi` |
| Radios et Bluetooth | `Devices_Radios`, `Devices_Bluetooth`, `Devices_Enumeration`, `Foundation`, `Foundation_Collections` |
| GPU | `Win32_Graphics_Dxgi`, `Win32_System_Performance` |
| Presse-papier | `Win32_System_DataExchange`, `Win32_System_Memory`, `Win32_System_Ole`, `Win32_UI_Input_KeyboardAndMouse`, `Win32_UI_Accessibility`, `Win32_System_Threading`, `Win32_System_LibraryLoader` |
| Lanceur et icônes | `Win32_UI_Shell`, `Win32_UI_Shell_Common`, `Win32_System_Com`, `Win32_Storage_FileSystem` |
| Média | `Media_Control`, `Storage_Streams` |
| Notifications | `UI_Notifications`, `UI_Notifications_Management`, `ApplicationModel` |

## Conventions de contribution

- Un commit par petit changement, avec un message court en anglais à l'impératif (par exemple « Add timer chip to the idle pill »).
- Une branche par fonctionnalité, nommée `feature/...`, jamais de travail direct sur `main`.
- La logique pure est testée (classification du presse-papier, recherche floue, réglages, machine à états du minuteur). Lance `cargo test --lib` et `pnpm build` avant de proposer un changement.

## Limitations connues

- Windows uniquement : le code s'appuie sur les API natives de Windows.
- Les yeux de la mascotte « Soul » ne suivent pas encore la souris.
- La recherche du lanceur ne couvre que les applications, le menu Démarrer et quelques dossiers de ton profil, pas tout le disque.
- La compilation de production (`pnpm tauri build`) n'a pas encore été vérifiée.
- Aucune licence n'est définie pour l'instant.
