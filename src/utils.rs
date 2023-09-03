use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SEMVER_REGEX: Regex = Regex::new(r"([0-9]+.[0-9]+.[0-9]+)(-[0-9a-zA-Z_]+)?").unwrap();
}

lazy_static! {
    static ref GIT_PROJECT_URL_REGEX: Regex = Regex::new(r"https?://([0-9a-zA-Z/._-]+)\.git").unwrap();
}

lazy_static! {
    static ref GITHUB_PROJECT_REGEX: Regex =
        Regex::new(r"https?://github.com/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref GITLAB_PROJECT_REGEX: Regex =
        Regex::new(r"https?://gitlab.com/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref GNOME_GITLAB_PROJECT_REGEX: Regex =
        Regex::new(r"https?://gitlab.gnome.org/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref PAGURE_PROJECT_REGEX: Regex = Regex::new(r"https://pagure.io/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref GNU_PROJECT_REGEX: Regex =
        Regex::new(r"https?://ftp.gnu.org/(?:pub/)?gnu/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref NONGNU_RELEASE_REGEX: Regex =
        Regex::new(r"https?://download.savannah.nongnu.org/releases/([0-9a-zA-Z_-]+)").unwrap();
}
lazy_static! {
    static ref NONGNU_PROJECT_REGEX: Regex =
        Regex::new(r"https?://savannah.nongnu.org/(?:download|projects)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref BITBUCKET_PROJECT_REGEX: Regex =
        Regex::new(r"https?://bitbucket.org/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

pub fn get_git_url_from_generic_url(generic_url: &str) -> Option<String> {
    if let Some(git_url) = get_github_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_gitlab_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_gnome_gitlab_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_pagure_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_gnu_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_nongnu_release_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_nongnu_project_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_bitbucket_url_from_generic_url(generic_url) {
        return Some(git_url);
    }
    // The SourceForge git access is documented here
    // https://sourceforge.net/p/forge/documentation/Git/#anonymous-access-read-only
    None
}

pub fn get_github_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match GITHUB_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let user_name: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!("https://github.com/{}/{}.git", user_name, project_name));
}

pub fn get_gitlab_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match GITLAB_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let user_name: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!("https://gitlab.com/{}/{}.git", user_name, project_name));
}

pub fn get_gnome_gitlab_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match GNOME_GITLAB_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let user_name: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!(
        "https://gitlab.gnome.org/{}/{}.git",
        user_name, project_name
    ));
}

pub fn get_pagure_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match PAGURE_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!("https://pagure.io/{}.git", project_name));
}

pub fn get_gnu_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match GNU_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!("https://git.savannah.gnu.org/git/{}.git", project_name));
}

pub fn get_nongnu_release_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match NONGNU_RELEASE_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!(
        "https://git.savannah.nongnu.org/git/{}.git",
        project_name
    ));
}

pub fn get_nongnu_project_url_from_generic_url(generic_url: &str) -> Option<String> {
    let captured_groups = match NONGNU_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!(
        "https://git.savannah.nongnu.org/git/{}.git",
        project_name
    ));
}

pub fn get_bitbucket_url_from_generic_url(generic_url: &str) -> Option<String> {
    // Bitbucket does not allow anonymous git access by default, so this
    // might fail.
    let captured_groups = match BITBUCKET_PROJECT_REGEX.captures(generic_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let username: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!("https://bitbucket.org/{}/{}.git", username, project_name));
}

pub fn get_semver_from_archive_url(archive_url: &str) -> Option<String> {
    let archive_filename = archive_url.split("/").last().unwrap();
    let captured_groups = match SEMVER_REGEX.captures(archive_filename) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    return Some(captured_groups[1].to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_git_url_from_generic_url() {
        let git_url =
            crate::utils::get_git_url_from_generic_url("https://github.com/sass/libsass/archive/3.6.4.tar.gz");
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://github.com/sass/libsass.git");

        let git_url = crate::utils::get_git_url_from_generic_url("https://github.com/sass/libsass");
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://github.com/sass/libsass.git");

        let git_url = crate::utils::get_git_url_from_generic_url(
            "https://gitlab.com/rszibele/e-juice-calc/-/archive/1.0.7/e-juice-calc-1.0.7.tar.bz2",
        );
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://gitlab.com/rszibele/e-juice-calc.git");

        let git_url = crate::utils::get_git_url_from_generic_url("https://gitlab.com/rszibele/e-juice-calc");
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://gitlab.com/rszibele/e-juice-calc.git");

        let git_url = crate::utils::get_git_url_from_generic_url(
            "https://gitlab.gnome.org/GNOME/libsecret/-/archive/0.19.1/libsecret-0.19.1.tar.gz",
        );
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://gitlab.gnome.org/GNOME/libsecret.git");

        let git_url = crate::utils::get_git_url_from_generic_url("https://gitlab.gnome.org/GNOME/libsecret");
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://gitlab.gnome.org/GNOME/libsecret.git");

        let git_url = crate::utils::get_git_url_from_generic_url(
            "https://pagure.io/libaio/archive/libaio-0.3.111/libaio-libaio-0.3.111.tar.gz",
        );
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://pagure.io/libaio.git");

        let git_url = crate::utils::get_git_url_from_generic_url(
            "https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.16.tar.gz",
        );
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://git.savannah.gnu.org/git/libiconv.git");

        let git_url =
            crate::utils::get_git_url_from_generic_url("http://ftp.gnu.org/gnu/autoconf/autoconf-2.13.tar.gz");
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://git.savannah.gnu.org/git/autoconf.git");

        let git_url = crate::utils::get_git_url_from_generic_url(
            "https://download.savannah.nongnu.org/releases/openexr/openexr-2.2.1.tar.gz",
        );
        assert!(git_url.is_some());
        assert_eq!(
            git_url.unwrap(),
            "https://git.savannah.nongnu.org/git/openexr.git"
        );

        let git_url = crate::utils::get_git_url_from_generic_url(
            "http://savannah.nongnu.org/download/icoutils/icoutils-0.31.1.tar.bz2",
        );
        assert!(git_url.is_some());
        assert_eq!(
            git_url.unwrap(),
            "https://git.savannah.nongnu.org/git/icoutils.git"
        );

        let git_url = crate::utils::get_git_url_from_generic_url("https://savannah.nongnu.org/projects/acl");
        assert!(git_url.is_some());
        assert_eq!(git_url.unwrap(), "https://git.savannah.nongnu.org/git/acl.git");

        let git_url = crate::utils::get_git_url_from_generic_url(
            "https://bitbucket.org/Doomseeker/doomseeker/get/1.3.1.tar.bz2",
        );
        assert!(git_url.is_some());
        assert_eq!(
            git_url.unwrap(),
            "https://bitbucket.org/Doomseeker/doomseeker.git"
        );
    }
    #[test]
    pub fn test_get_semver_from_archive() {
        let version = crate::utils::get_semver_from_archive_url(
            "https://download-fallback.gnome.org/sources/libgda/5.2/libgda-5.2.9.tar.xz",
        );
        assert!(version.is_some());
        assert_eq!(version.unwrap(), "5.2.9");

        let version = crate::utils::get_semver_from_archive_url(
            "https://download.gnome.org/core/3.28/3.28.2/sources/libgsf-1.14.43.tar.xz",
        );
        assert!(version.is_some());
        assert_eq!(version.unwrap(), "1.14.43");

        let version = crate::utils::get_semver_from_archive_url(
            "https://download.gnome.org/core/3.28/3.28.2/sources/libgsf-1.14.43.tar.xz",
        );
        assert!(version.is_some());
        assert_eq!(version.unwrap(), "1.14.43");

        let version = crate::utils::get_semver_from_archive_url(
  "https://github.com/haskell/ghc/releases/download/ghc-8.6.3-release/ghc-8.6.3-armv7-deb8-linux.tar.xz"
);
        assert!(version.is_some());
        assert_eq!(version.unwrap(), "8.6.3");

        let version = crate::utils::get_semver_from_archive_url(
            "https://github.com/GNOME/libxml2/archive/v2.9.10.tar.gz",
        );
        assert!(version.is_some());
        assert_eq!(version.unwrap(), "2.9.10");

        let version =
            crate::utils::get_semver_from_archive_url("https://github.com/sass/libsass/archive/3.6.4.tar.gz");
        assert!(version.is_some());
        assert_eq!(version.unwrap(), "3.6.4");
    }
}
