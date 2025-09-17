use cucumber::then;

use crate::world::World;

fn parse_list(s: &str) -> Vec<String> {
    s.split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[then(regex = r#"^the vector equals \"([^\"]*)\"$"#)]
pub async fn vec_equals(world: &mut World, list: String) {
    let mut expect = parse_list(&list);
    let mut got = world.last_vec.clone();
    expect.sort();
    expect.dedup();
    got.sort();
    got.dedup();
    assert_eq!(got, expect);
}

#[then(regex = r#"^the vector contains \"([^\"]*)\"$"#)]
pub async fn vec_contains(world: &mut World, list: String) {
    let expect = parse_list(&list);
    for e in expect {
        assert!(
            world.last_vec.contains(&e),
            "missing {} in {:?}",
            e,
            world.last_vec
        );
    }
}

#[then(regex = r"^the result is Ok$")]
pub async fn result_ok(world: &mut World) {
    let r = world.last_result.as_ref().expect("no result");
    assert!(r.is_ok(), "expected Ok, got {:?}", r);
}

#[then(regex = r#"^the result is Err with missing \"([^\"]*)\"$"#)]
pub async fn result_err_missing(world: &mut World, list: String) {
    let r = world.last_result.as_ref().expect("no result");
    assert!(r.is_err(), "expected Err, got Ok");
    let miss = r.as_ref().err().unwrap();
    let expect = parse_list(&list);
    for e in expect {
        assert!(miss.contains(&e), "expected missing {:?} in {:?}", e, miss);
    }
}
