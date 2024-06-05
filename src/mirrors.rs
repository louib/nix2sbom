use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    // This mapping is taken from
    // https://github.com/NixOS/nixpkgs/blob/454c26e063321ff2229bf1dfedab4a8f80e60008/pkgs/build-support/fetchurl/mirrors.nix
    // The translation is not happening when extracting all the derivations, so we have to do the
    // translation manually using this mapping. Instead of using the most efficient mirror, we pick
    // that one that better semantically describes the source of the package (the most
    // "authoritative" mirror).
    static ref MIRRORS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("hashedMirrors", "https://tarballs.nixos.org");
        m.insert("alsa", "https://www.alsa-project.org/files/pub/");
        m.insert("apache", "https://dlcdn.apache.org/");
        m.insert("bioc", "http://bioc.ism.ac.jp/");
        m.insert("cran", "https://cran.r-project.org/src/contrib/");
        m.insert("bitlbee", "https://get.bitlbee.org/");
        m.insert("gcc", "https://mirror.koddos.net/gcc/");
        m.insert("gnome", "https://download.gnome.org/");
        m.insert("gnu", "https://ftp.gnu.org/pub/gnu/");
        m.insert("gnupg", "https://gnupg.org/ftp/gcrypt/");
        m.insert("ibiblioPubLinux", "https://www.ibiblio.org/pub/Linux/");
        m.insert("imagemagick", "https://www.imagemagick.org/download/");
        m.insert("kde", "https://cdn.download.kde.org/");
        m.insert("kernel", "https://cdn.kernel.org/pub/");
        m.insert("mysql", "https://cdn.mysql.com/Downloads/");
        m.insert("maven", "https://repo1.maven.org/maven2/");
        m.insert("mozilla", "https://download.cdn.mozilla.net/pub/mozilla.org/");
        m.insert("osdn", "https://osdn.dl.osdn.jp/");
        m.insert("postgresql", "https://ftp.postgresql.org/pub/");
        m.insert("qt", "https://download.qt.io/");
        m.insert("sageupstream", "https://mirrors.mit.edu/sage/spkg/upstream/");
        m.insert("samba", "https://www.samba.org/ftp/");
        m.insert("savannah", "https://ftp.gnu.org/gnu/");
        m.insert("sourceforge", "https://downloads.sourceforge.net/");
        m.insert("steamrt", "https://repo.steampowered.com/steamrt/");
        m.insert("tcsh", "https://astron.com/pub/tcsh/");
        m.insert("xfce", "https://archive.xfce.org/");
        m.insert("xorg", "https://xorg.freedesktop.org/releases/");
        m.insert("cpan", "https://cpan.metacpan.org/");
        m.insert("hackage", "https://hackage.haskell.org/package/");
        m.insert("luarocks", "https://luarocks.org/");
        m.insert("pypi", "https://pypi.io/packages/source/");
        m.insert("testpypi", "https://test.pypi.io/packages/source/");
        m.insert("centos", "https://vault.centos.org/");
        m.insert("debian", "https://httpredir.debian.org/debian/");
        m.insert("fedora", "https://archives.fedoraproject.org/pub/fedora/");
        m.insert("gentoo", "https://distfiles.gentoo.org/");
        m.insert("opensuse", "https://opensuse.hro.nl/opensuse/distribution/");
        m.insert("ubuntu", "https://nl.archive.ubuntu.com/ubuntu/");
        m.insert("openbsd", "https://ftp.openbsd.org/pub/OpenBSD/");
        m
    };
    static ref MIRROR_URL_REGEX: Regex =
        Regex::new(r"mirror://([0-9a-zA-Z_-]+)/(.*)?").unwrap();
}
lazy_static! {}

pub fn translate_url(url: &str) -> String {
    if !url.starts_with("mirror://") {
        return url.to_string();
    }
    if let Some(g) = MIRROR_URL_REGEX.captures(url) {
        if g.len() == 0 {
            return url.to_string();
        }

        let mirror_name = &g[1];
        if let Some(mirror_url) = MIRRORS.get(mirror_name) {
            return url.replace(&format!("mirror://{}/", mirror_name), mirror_url);
        } else {
            panic!("Unknown mirror name: {}", mirror_name);
        }
    }
    return url.to_string();
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_translate_mirror_url() {
        let url = crate::mirrors::translate_url("https://github.com/sass/libsass/archive/3.6.4.tar.gz");
        assert_eq!(url, "https://github.com/sass/libsass/archive/3.6.4.tar.gz");

        let url = crate::mirrors::translate_url("mirror://gnu/autoconf/autoconf-2.72.tar.xz");
        assert_eq!(url, "https://ftp.gnu.org/pub/gnu/autoconf/autoconf-2.72.tar.xz");
    }
}
