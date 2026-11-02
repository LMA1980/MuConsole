// SPDX-License-Identifier: AGPL-3.0-only
//! Hash router (`#/{area}/{owner}/{repo}/{submodule?}/{branch}/{job}`),
//! legacy `#/jobs/{id}` redirect, `Esc` up-one-level navigation and
//! breakpoint-aware C2 breadcrumb segment builder.

use muconsole_layout::Breakpoint;
use muconsole_protocol::{Area, RepoId, Route, SubmoduleId};

/// Short URL slug for [`Area`].
#[must_use]
pub const fn area_slug(area: Area) -> &'static str {
    match area {
        Area::Overview => "ovr",
        Area::Ci => "ci",
        Area::Cd => "cd",
        Area::Fleet => "flt",
    }
}

/// Uppercase display label for [`Area`].
#[must_use]
pub const fn area_label(area: Area) -> &'static str {
    match area {
        Area::Overview => "OVR",
        Area::Ci => "CI",
        Area::Cd => "CD",
        Area::Fleet => "FLT",
    }
}

/// Parse an area slug (`ovr|ci|cd|flt` or full name).
#[must_use]
pub fn parse_area(s: &str) -> Option<Area> {
    match s.to_ascii_lowercase().as_str() {
        "ovr" | "overview" => Some(Area::Overview),
        "ci" => Some(Area::Ci),
        "cd" => Some(Area::Cd),
        "flt" | "fleet" => Some(Area::Fleet),
        _ => None,
    }
}

fn enc(s: &str) -> String {
    s.replace('/', "%2F")
}

fn dec(s: &str) -> String {
    s.replace("%2F", "/").replace("%2f", "/")
}

/// Parse a location hash (`#/...`) into a [`Route`].
///
/// Legacy `#/jobs/{id}` links redirect to `#/ci/_/_/main/{id}`.
#[must_use]
pub fn parse_hash(hash: &str) -> Route {
    let trimmed = hash.trim().trim_start_matches('#').trim_matches('/');
    if trimmed.is_empty() {
        return Route {
            area: Area::Overview,
            repo: None,
            submodule: None,
            branch: None,
            job: None,
        };
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.first().copied() == Some("jobs") {
        let job = parts.get(1).and_then(|s| s.parse::<u64>().ok());
        return Route {
            area: Area::Ci,
            repo: Some(RepoId::new("_", "_")),
            submodule: None,
            branch: Some("main".into()),
            job,
        };
    }
    let area = parts
        .first()
        .and_then(|s| parse_area(s))
        .unwrap_or(Area::Overview);
    let repo = match (parts.get(1), parts.get(2)) {
        (Some(&o), Some(&r)) if !o.is_empty() && !r.is_empty() => {
            Some(RepoId::new(&dec(o), &dec(r)))
        }
        _ => None,
    };
    let submodule = parts
        .get(3)
        .filter(|s| !s.is_empty() && **s != "-")
        .map(|s| SubmoduleId { path: dec(s) });
    let branch = parts
        .get(4)
        .filter(|s| !s.is_empty() && **s != "-")
        .map(|s| dec(s));
    let job = parts.get(5).and_then(|s| s.parse::<u64>().ok());
    Route {
        area,
        repo,
        submodule,
        branch,
        job,
    }
}

/// Format a [`Route`] as a canonical `#/...` location hash.
#[must_use]
pub fn format_hash(route: &Route) -> String {
    let mut out = format!("#/{}", area_slug(route.area));
    let Some(repo) = route.repo.as_ref() else {
        return out;
    };
    out.push('/');
    out.push_str(&enc(&repo.owner));
    out.push('/');
    out.push_str(&enc(&repo.repo));
    if route.submodule.is_none() && route.branch.is_none() && route.job.is_none() {
        return out;
    }
    out.push('/');
    match route.submodule.as_ref() {
        Some(sub) => out.push_str(&enc(&sub.path)),
        None => out.push('-'),
    }
    if route.branch.is_none() && route.job.is_none() {
        return out;
    }
    out.push('/');
    match route.branch.as_deref() {
        Some(b) => out.push_str(&enc(b)),
        None => out.push('-'),
    }
    if let Some(j) = route.job {
        out.push('/');
        out.push_str(&j.to_string());
    }
    out
}

/// Step up one hierarchy level on `Esc`:
/// `job -> branch -> submodule -> repo -> Overview`.
#[must_use]
pub fn escape_up(route: &Route) -> Route {
    let mut next = route.clone();
    if next.job.is_some() {
        next.job = None;
    } else if next.branch.is_some() {
        next.branch = None;
    } else if next.submodule.is_some() {
        next.submodule = None;
    } else if next.repo.is_some() {
        next.repo = None;
    } else {
        next.area = Area::Overview;
    }
    next
}

/// One segment of the C2 Breadcrumb bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbCrumb {
    pub id: &'static str,
    pub label: &'static str,
    pub value: String,
}

/// Build the C2 Breadcrumb segments for `route` and `breakpoint`, collapsing
/// middle segments to `…` on compact/phone viewports when needed.
#[must_use]
pub fn breadcrumb_segments(route: &Route, bp: Breakpoint) -> Vec<BreadcrumbCrumb> {
    let mut crumbs = vec![BreadcrumbCrumb {
        id: "c2-area",
        label: "AREA",
        value: area_label(route.area).to_owned(),
    }];
    if let Some(repo) = route.repo.as_ref() {
        crumbs.push(BreadcrumbCrumb {
            id: "c2-repo",
            label: "REPO",
            value: format!("{}/{}", repo.owner, repo.repo),
        });
    }
    if let Some(sub) = route.submodule.as_ref() {
        crumbs.push(BreadcrumbCrumb {
            id: "c2-submodule",
            label: "SUB",
            value: sub.path.clone(),
        });
    }
    if let Some(branch) = route.branch.as_ref() {
        crumbs.push(BreadcrumbCrumb {
            id: "c2-branch",
            label: "REF",
            value: branch.clone(),
        });
    }
    if let Some(job) = route.job {
        crumbs.push(BreadcrumbCrumb {
            id: "c2-job",
            label: "JOB",
            value: format!("#{job}"),
        });
    }
    if matches!(bp, Breakpoint::Phone | Breakpoint::CompactLandscape) && crumbs.len() > 3 {
        let last = crumbs.pop().expect("len > 3");
        crumbs.truncate(2);
        crumbs.push(BreadcrumbCrumb {
            id: "c2-ellipsis",
            label: "",
            value: "…".into(),
        });
        crumbs.push(last);
    }
    crumbs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_round_trips_and_redirects_legacy_jobs() {
        let r = Route {
            area: Area::Ci,
            repo: Some(RepoId::new("LMA1980", "MuConsole")),
            submodule: Some(SubmoduleId {
                path: "code/engine".into(),
            }),
            branch: Some("feat/rvv".into()),
            job: Some(42),
        };
        let h = format_hash(&r);
        assert_eq!(h, "#/ci/LMA1980/MuConsole/code%2Fengine/feat%2Frvv/42");
        assert_eq!(parse_hash(&h), r);

        // Legacy #/jobs/{id} redirect.
        let legacy = parse_hash("#/jobs/9876");
        assert_eq!(legacy.area, Area::Ci);
        assert_eq!(legacy.branch.as_deref(), Some("main"));
        assert_eq!(legacy.job, Some(9876));
    }

    #[test]
    fn escape_steps_up_hierarchy_and_breadcrumbs_collapse_on_phone() {
        let mut r = parse_hash("#/ci/LMA1980/MuConsole/code%2Fengine/feat%2Frvv/42");
        let phone = breadcrumb_segments(&r, Breakpoint::Phone);
        assert!(phone.iter().any(|c| c.value == "…"));

        r = escape_up(&r);
        assert!(r.job.is_none() && r.branch.is_some());
        r = escape_up(&r);
        assert!(r.branch.is_none() && r.submodule.is_some());
        r = escape_up(&r);
        assert!(r.submodule.is_none() && r.repo.is_some());
        r = escape_up(&r);
        assert!(r.repo.is_none());
    }
}
