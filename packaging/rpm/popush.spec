Name:           popush
Version:        1.0.0
Release:        1%{?dist}
Summary:        Your VPS, one click away.

License:        GPL-3.0-only
URL:            https://popush.dev
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  nodejs
BuildRequires:  pnpm
BuildRequires:  webkit2gtk4.1-devel
BuildRequires:  gtk3-devel
BuildRequires:  libappindicator-gtk3-devel
BuildRequires:  librsvg2-devel
BuildRequires:  openssl-devel
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib

Requires:       webkit2gtk4.1
Requires:       gtk3
Requires:       librsvg2

%description
Popush is a desktop tool for deploying local git repositories to your own
servers over SSH. Pick a repository, choose a host, and push your changes
without hand-writing deploy scripts or juggling terminal sessions.

Local repository access is requested per-repository rather than through broad
home-directory access, because a security-focused tool should not ask for more
than it needs.

Popush is developed by twostep.

%prep
%autosetup -n %{name}-%{version}

%build
pnpm install --frozen-lockfile
pnpm tauri build

%install
# Binary
install -Dm0755 src-tauri/target/release/popush %{buildroot}%{_bindir}/popush

# Desktop entry
install -Dm0644 packaging/desktop/dev.popush.Popush.desktop \
    %{buildroot}%{_datadir}/applications/dev.popush.Popush.desktop

# AppStream metainfo
install -Dm0644 packaging/flatpak/dev.popush.Popush.metainfo.xml \
    %{buildroot}%{_datadir}/metainfo/dev.popush.Popush.metainfo.xml

# Scalable icon
install -Dm0644 packaging/desktop/icons/dev.popush.Popush.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/dev.popush.Popush.svg

# Raster icons
for size in 16 24 32 48 64 128 256 512; do
    install -Dm0644 "packaging/desktop/icons/dev.popush.Popush-${size}.png" \
        "%{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/dev.popush.Popush.png"
done

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/dev.popush.Popush.desktop
appstream-util validate-relax --nonet \
    %{buildroot}%{_datadir}/metainfo/dev.popush.Popush.metainfo.xml

%files
%license LICENSE
%{_bindir}/popush
%{_datadir}/applications/dev.popush.Popush.desktop
%{_datadir}/metainfo/dev.popush.Popush.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/dev.popush.Popush.svg
%{_datadir}/icons/hicolor/*/apps/dev.popush.Popush.png

%changelog
* Sat Jul 12 2026 twostep <hello@popush.dev> - 1.0.0-1
- First stable release.
