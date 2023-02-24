pub use bitbuffer::Result as ReadResult;

pub use pyo3::prelude::*;
pub use pyo3::wrap_pyfunction;
pub use pyo3::exceptions::PyOSError;
pub use std::env;
pub use std::fs;

pub use crate::demo::{
    message::MessageType,
    parser::{
        DemoParser, GameEventError, MatchState, MessageTypeAnalyser, Parse, ParseError,
        ParserState, Result,
    },
    parser::player_summary_analyzer::PlayerSummaryAnalyzer,
    Demo, Stream,
};

pub use crate::demo::parser::analyser::Class;

pub(crate) mod consthash;
pub mod demo;
pub(crate) mod nullhasher;

#[cfg(test)]
#[track_caller]
fn test_roundtrip_write<
    'a,
    T: bitbuffer::BitRead<'a, bitbuffer::LittleEndian>
        + bitbuffer::BitWrite<bitbuffer::LittleEndian>
        + std::fmt::Debug
        + std::cmp::PartialEq,
>(
    val: T,
) {
    let mut data = Vec::with_capacity(128);
    use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
    let pos = {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);
        val.write(&mut stream).unwrap();
        stream.bit_len()
    };

    let mut read = BitReadStream::new(BitReadBuffer::new_owned(data, LittleEndian));
    assert_eq!(
        val,
        read.read().unwrap(),
        "Failed to assert the parsed message is equal to the original"
    );
    assert_eq!(
        pos,
        read.pos(),
        "Failed to assert that all encoded bits ({}) are used for decoding ({})",
        pos,
        read.pos()
    );
}

#[cfg(test)]
#[track_caller]
fn test_roundtrip_encode<
    'a,
    T: Parse<'a> + crate::demo::parser::Encode + std::fmt::Debug + std::cmp::PartialEq,
>(
    val: T,
    state: &ParserState,
) {
    let mut data = Vec::with_capacity(128);
    use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
    let pos = {
        let mut stream = BitWriteStream::new(&mut data, LittleEndian);
        val.encode(&mut stream, state).unwrap();
        stream.bit_len()
    };

    let mut read = BitReadStream::new(BitReadBuffer::new_owned(data, LittleEndian));
    pretty_assertions::assert_eq!(
        val,
        T::parse(&mut read, state).unwrap(),
        "Failed to assert the parsed message is equal to the original"
    );
    pretty_assertions::assert_eq!(
        pos,
        read.pos(),
        "Failed to assert that all encoded bits ({}) are used for decoding ({})",
        pos,
        read.pos()
    );
}

#[pyfunction(name="main")]
fn py_main(path: String) -> PyResult<String> {
    let file = fs::read(path)?;
    let demo = Demo::new(&file);

    let parser = DemoParser::new_all_with_analyser(demo.get_stream(), PlayerSummaryAnalyzer::new());
    let (header, state) = parser.parse()?;

    let parser = DemoParser::new_all(demo.get_stream()); // TODO: remove once new_all_with_analyser properly reports team, class, etc
    let (_, all_state) = parser.parse()?;

    let table_header = "player,id,points,kills,deaths,assists,destruction,captures,defenses,domination,revenge,ubers,headshots,teleports,healing,backstabs,bonus,support,damage,team,class";

    let mut data: String = "".to_owned();
    for (user_id, user_data) in state.users {
        let player_name = user_data.name;
        let team = all_state.users[&user_id].team;
        let mut tf_class = Class::Other;
        // get the class with most spawns
        for (c, _s) in all_state.users[&user_id].classes.sorted() {
            tf_class = c;
            break;
        }
        let steam_id = user_data.steam_id;
        let summary = state.player_summaries.get(&user_id);
        match summary {
            Some(s) => {
                let s: String = format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    player_name.replace(",", ""),
                    steam_id,
                    s.points,
                    s.kills,
                    s.deaths,
                    s.assists,
                    s.buildings_destroyed,
                    s.captures,
                    s.defenses,
                    s.dominations,
                    s.revenges,
                    s.ubercharges,
                    s.headshots,
                    s.teleports,
                    s.healing,
                    s.backstabs,
                    s.bonus_points,
                    s.support,
                    s.damage_dealt,
                    team,
                    tf_class,
                );
                data = data + &s;
            },
            None => {
                // No summary for this player - they likely joined at the end of the match, or left before they did anything noteworthy
            }
        }
    }

    Ok(format!("{:?}", header) + "\n" + table_header + "\n" + &data)
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
#[pyo3(name = "tf_demo_parser")]
fn tf_demo_parser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}