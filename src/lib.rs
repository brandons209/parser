pub use bitbuffer::Result as ReadResult;

pub use pyo3::prelude::*;
pub use pyo3::wrap_pyfunction;
pub use pyo3::exceptions::PyOSError;
pub use std::env;
pub use std::fs;

pub use crate::demo::{
    message::MessageType,
    parser::{
        DemoParser, Parse, ParserState, ParseError, Result,
        GameEventError,
    },
    parser::analyser::Analyser,
    parser::analyser::UserId,
    parser::player_summary_analyzer::PlayerSummaryAnalyzer,
    parser::gamestateanalyser::GameStateAnalyser,
    parser::gamestateanalyser::BuildingClass,
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

    // gets scoreboard information
    let parser = DemoParser::new_all_with_analyser(demo.get_stream(), PlayerSummaryAnalyzer::new());
    let (header, player_state) = parser.parse()?;

    // gets building events, kill events, team, and class information
    let parser = DemoParser::new_all_with_analyser(demo.get_stream(), GameStateAnalyser::new());
    let (_, mut game_state) = parser.parse()?;

    // gets chat messages, deaths, rounds, and start_tick
    let parser = DemoParser::new_all_with_analyser(demo.get_stream(), Analyser::new());
    let (_, server_state) = parser.parse()?;

    // process scoreboard information
    let scoreboard_table_header = "player,id,points,kills,deaths,assists,destruction,captures,defenses,domination,revenge,ubers,headshots,teleports,healing,backstabs,bonus,support,damage,team,class";
    let mut scoreboard_data: String = "".to_owned();
    for (user_id, user_data) in &player_state.users {
        let player_name = &user_data.name;
        let player_data = game_state.get_or_create_player(user_data.entity_id);
        let team = player_data.team;
        let tf_class = player_data.class;
        let steam_id = &user_data.steam_id;
        let summary = player_state.player_summaries.get(&user_id);
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
                scoreboard_data = scoreboard_data + &s;
            },
            None => {
                // No summary for this player - they likely joined at the end of the match, or left before they did anything noteworthy
            }
        }
    }

    // process building information
    
    let common_building_table_header = "id,builder,x,y,z,level,max_health,health,sapped,team,angle,building".to_owned();
    let sentry_table_header = common_building_table_header.clone() + ",player_controlled,target,shells,rockets,is_mini";
    let dispenser_table_header = common_building_table_header.clone() + ",metal";
    let teleporter_table_header = common_building_table_header.clone() + ",is_entrance,connected_to,recharge_time,recharge_duration,times_used,yaw_to_exit";

    let mut sentry_data: String = "".to_owned();
    let mut dispenser_data: String = "".to_owned();
    let mut teleporter_data: String = "".to_owned();

    for (entity_id, building_data) in game_state.buildings {
        let mut common = "".to_owned();
        let mut specific = "".to_owned();

        let builder;
        
        match player_state.users.get(&building_data.builder()) {
            Some(info) => {
                builder = info.steam_id.clone();
            },
            None => {
                builder = "unknown".to_string();
            }
        }

        let s: String = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            entity_id,
            builder,
            building_data.position().x,
            building_data.position().y,
            building_data.position().z,
            building_data.level(),
            building_data.max_health(),
            building_data.health(),
            building_data.sapped(),
            building_data.team(),
            building_data.angle(),
            building_data.building(),
        );

        common = common + &s;
        
        if building_data.class() == BuildingClass::Sentry
        {
            let target;

            match player_state.users.get(&building_data.auto_aim_target()) {
                Some(info) => {
                    target = info.steam_id.clone();
                },
                None => {
                    target = "unknown".to_string();
                }
            }

            let s: String = format!(
                ",{},{},{},{},{}\n",
                building_data.player_controlled(),
                target,
                building_data.shells(),
                building_data.rockets(),
                building_data.is_mini(),
            );
            specific = specific + &s;
 
            common = common + &specific;
            sentry_data = sentry_data + &common;
        }
        else if building_data.class() == BuildingClass::Dispenser
        {
            let s: String = format!(
                ",{}\n",
                building_data.metal(),
            );
            specific = specific + &s;
   
            common = common + &specific;
            dispenser_data = dispenser_data + &common;
        }
        else if building_data.class() == BuildingClass::Teleporter
        {
            let s: String = format!(
                ",{},{},{},{},{},{}\n",
                building_data.is_entrance(),
                building_data.other_end(),
                building_data.recharge_time(),
                building_data.recharge_duration(),
                building_data.times_used(),
                building_data.yaw_to_exit(),
            );
            specific = specific + &s;
        
            common = common + &specific;
            teleporter_data = teleporter_data + &common;
        }
    }

    let kills_table_header = "tick,attacker,assister,victim,weapon";
    let mut kill_data: String = "".to_owned();
    for kill in game_state.kills {
        let killer;
        let killed;
        let assister;

        match player_state.users.get(&UserId::from(kill.attacker_id)) {
            Some(info) => {
                killer = info.steam_id.clone();
            },
            None => {
                killer = "unknown".to_string();
            }
        }
        match player_state.users.get(&UserId::from(kill.assister_id)) {
            Some(info) => {
                assister = info.steam_id.clone();
            },
            None => {
                assister = "unknown".to_string();
            }
        }
        match player_state.users.get(&UserId::from(kill.victim_id)) {
            Some(info) => {
                killed = info.steam_id.clone();
            },
            None => {
                killed = "unknown".to_string();
            }
        }

        let s: String = format!(
            "{},{},{},{},{}\n",
            kill.tick,
            killer,
            assister,
            killed,
            kill.weapon,
        );
        kill_data = kill_data + &s;
    }


    let rounds_table_header = "end_tick,length,winner";
    let mut rounds_data: String = "".to_owned();
    for round in server_state.rounds {
        let s: String = format!(
            "{},{},{}\n",
            round.end_tick,
            round.length,
            round.winner,
        );
        rounds_data = rounds_data + &s;
    }

    Ok(format!("{:?}", header) + "\n" + 
        scoreboard_table_header + "\n" + 
        &scoreboard_data  + "\n[=============]\n" +  
        &sentry_table_header + "\n" + 
        &sentry_data  + "\n[=============]\n" +  
        &dispenser_table_header + "\n" + 
        &dispenser_data  + "\n[=============]\n" +  
        &teleporter_table_header + "\n" + 
        &teleporter_data  + "\n[=============]\n" +  
        kills_table_header + "\n" + 
        &kill_data  + "\n[=============]\n" +  
        rounds_table_header + "\n" + 
        &rounds_data)
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