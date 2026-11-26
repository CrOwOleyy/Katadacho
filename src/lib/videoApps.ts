/** Application au premier plan : nom de son exécutable et titre de sa fenêtre. */
export interface ForegroundApp {
  process: string;
  title: string;
}

/** Lecteurs vidéo installés, reconnus à leur exécutable Windows. */
const VIDEO_PROCESSES = [
  "vlc.exe",
  "mpv.exe",
  "mpc-hc.exe",
  "mpc-hc64.exe",
  "mpc-be64.exe",
  "potplayermini64.exe",
  "wmplayer.exe",
  "kodi.exe",
];

/**
 * Services de streaming, reconnus au titre de la fenêtre : onglet du navigateur ou application.
 * Le titre d'un navigateur finit par son nom (« Vidéo - YouTube - Google Chrome »), d'où la recherche
 * d'un fragment plutôt que d'un titre exact.
 */
const VIDEO_TITLES = ["youtube", "netflix", "crunchyroll", "prime video", "disney+", "twitch"];

/** Vrai quand l'application au premier plan est un lecteur vidéo ou un site de streaming. */
export function isVideoApp(app: ForegroundApp): boolean {
  const process = app.process.trim().toLowerCase();
  const title = app.title.toLowerCase();
  return VIDEO_PROCESSES.includes(process) || VIDEO_TITLES.some((service) => title.includes(service));
}
