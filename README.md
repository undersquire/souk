# Souk - An Independent Flatpak App Store

Souk is a flatpak-based app store, written with GTK4 and Rust.
It's written from the ground up to be an app store that works both
on desktop and on mobile devices like the PinePhone, PineTab, and Librem 5.

![Screenshot of Souk's app details page](data/screenshots/app-details.png)

## Getting in Touch
If you have any questions regarding the use or development of Souk,
want to discuss design or simply hang out, please join us on our [#souk:gnome.org](https://matrix.to/#/#souk:gnome.org) channel.

## Building

### GNOME Builder

GNOME Builder is the environment used for developing this application. It can use Flatpak manifests to create a consistent building and running environment cross-distro. Thus, it is highly
recommended you use it.

1. Download [GNOME Builder](https://flathub.org/apps/details/org.gnome.Builder).
2. In Builder, click the "Clone Repository" button at the bottom, using `git@gitlab.gnome.org/haecker-felix/souk.git`
or `https://gitlab.gnome.org/haecker-felix/souk.git` as the URL.
3. Click the build button at the top once the project is loaded.


## Installation

Depending on how you want it installed instructions can differ. If you
used GNOME Builder to build it, clicking the bar at the top window will 
open a submenu with "Export Bundle". This will create a flatpak bundle, 
which can be installed on any system that supports flatpak.

## Conduct

Souk operates under the GNOME Code Of Conduct. See the full
text of the Code Of Conduct [here](CODE_OF_CONDUCT.md).