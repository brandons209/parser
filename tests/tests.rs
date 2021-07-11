use std::fs;
use test_case::test_case;

use tf_demo_parser::demo::parser::gamestateanalyser::{GameState, GameStateAnalyser};
use tf_demo_parser::{Demo, DemoParser, MatchState};

#[test_case("test_data/small.dem", "test_data/small.json"; "small.dem")]
#[test_case("test_data/gully.dem", "test_data/gully.json"; "gully.dem")]
#[test_case("test_data/comp.dem", "test_data/comp.json"; "comp.dem")]
#[test_case("test_data/malformed_cvar.dem", "test_data/malformed_cvar.json"; "malformed_cvar.dem")]
#[test_case("test_data/unicode-saytext.dem", "test_data/unicode-saytext.json"; "unicode-saytext.dem")]
#[test_case("test_data/nousers.dem", "test_data/nousers.json"; "nousers.dem")]
#[test_case("test_data/decal.dem", "test_data/decal.json"; "decal.dem")]
#[test_case("test_data/saytext2.dem", "test_data/saytext2.json"; "saytext2.dem")]
#[test_case("test_data/emptysaytext.dem", "test_data/emptysaytext.json"; "emptysaytext.dem")]
fn snapshot_test(input_file: &str, snapshot_file: &str) {
    let file = fs::read(input_file).expect("Unable to read file");
    let demo = Demo::new(&file);
    let (_, state) = DemoParser::new(demo.get_stream()).parse().unwrap();

    let expected: MatchState = serde_json::from_slice(
        fs::read(snapshot_file)
            .expect("Unable to read file")
            .as_slice(),
    )
    .unwrap();
    pretty_assertions::assert_eq!(expected, state);
}

#[test_case("test_data/small.dem", "test_data/small_game_state.json"; "small.dem")]
#[test_case("test_data/gully.dem", "test_data/gully_game_state.json"; "gully.dem")]
fn game_state_test(input_file: &str, snapshot_file: &str) {
    let file = fs::read(input_file).expect("Unable to read file");
    let demo = Demo::new(&file);
    let (_, state) = DemoParser::new_with_analyser(demo.get_stream(), GameStateAnalyser::new())
        .parse()
        .unwrap();

    let expected: GameState = serde_json::from_slice(
        fs::read(snapshot_file)
            .expect("Unable to read file")
            .as_slice(),
    )
    .unwrap();
    pretty_assertions::assert_eq!(expected, state);
}
