use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;

#[allow(clippy::needless_lifetimes)]
fn true_or_false<'a>(
    selected_bbs: &'a [bool],
    unselected: bool,
) -> impl Iterator<Item = usize> + Clone + 'a {
    let res = selected_bbs
        .iter()
        .enumerate()
        .filter(move |(_, is_selected)| unselected ^ **is_selected)
        .map(|(i, _)| i);
    res
}

#[allow(clippy::needless_lifetimes)]
pub fn true_indices<'a>(selected_bbs: &'a [bool]) -> impl Iterator<Item = usize> + Clone + 'a {
    true_or_false(selected_bbs, false)
}

pub fn natural_cmp(s1: &str, s2: &str) -> Ordering {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d+)").unwrap();
    }
    let mut idx = 0;
    while idx < s1.len().min(s2.len()) {
        let c1 = s1.chars().nth(idx).unwrap();
        let c2 = s2.chars().nth(idx).unwrap();
        if c1.is_ascii_digit() && c2.is_ascii_digit() {
            let n1 = RE.captures(&s1[idx..]).unwrap()[0]
                .parse::<usize>()
                .unwrap();
            let n2 = RE.captures(&s2[idx..]).unwrap()[0]
                .parse::<usize>()
                .unwrap();
            if n1 != n2 {
                return n1.cmp(&n2);
            }
            idx += n1.to_string().len();
        } else {
            if c1 != c2 {
                return c1.cmp(&c2);
            }
            idx += 1;
        }
    }
    s1.len().cmp(&s2.len())
}

#[test]
fn test_natural_sort() {
    assert_eq!(natural_cmp("s10", "s2"), Ordering::Greater);
    assert_eq!(natural_cmp("10s", "s2"), Ordering::Less);
    assert_eq!(natural_cmp("10", "2"), Ordering::Greater);
    assert_eq!(natural_cmp("10.0", "10.0"), Ordering::Equal);
    assert_eq!(natural_cmp("20.0", "10.0"), Ordering::Greater);
    assert_eq!(
        natural_cmp("a lot of text 20.0 .", "a lot of text 100.0"),
        Ordering::Less
    );
    assert_eq!(
        natural_cmp("a lot of 7text 20.0 .", "a lot of 3text 100.0"),
        Ordering::Greater
    );
}
