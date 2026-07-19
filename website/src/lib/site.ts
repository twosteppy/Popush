export const SITE = {
  github: 'https://github.com/twosteppy/Popush',
  releases: 'https://github.com/twosteppy/Popush/releases/latest',
  releasesAll: 'https://github.com/twosteppy/Popush/releases',
  readme: 'https://github.com/twosteppy/Popush#readme',
  // Current release + direct installer links. Bump `version` and these two
  // filenames when cutting a new release (the asset names carry the version).
  version: 'v1.0.1',
  download: {
    windows:
      'https://github.com/twosteppy/Popush/releases/download/v1.0.1/Popush_1.0.1_x64-setup.exe',
    linux:
      'https://github.com/twosteppy/Popush/releases/download/v1.0.1/Popush_1.0.1_amd64.AppImage',
  },
  // Shown in the chip; the copy button copies the real one-liner below.
  installDisplay: 'popush.dev/install',
  installCommand:
    'curl -fsSL https://raw.githubusercontent.com/twosteppy/Popush/main/get-popush.sh | bash',
} as const;
