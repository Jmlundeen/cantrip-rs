//! Every scenario of `scenarios/*.cantrip` played at a table (docs/scenarios-in-the-browser.md): Bot1's seat driven, its
//! every answer the one the host's step says the script means, sent back over the wire as a client would send it. A
//! scenario that passes alone must pass so: what the host says to answer is what the script means, in the seat's own
//! handles.

use std::path::{Path, PathBuf};

use cantrip_host::protocol::{ActionKindView, AnswerPayload, TakeActionAnswer, ViewHandle};
use cantrip_host::session::Session;
use cantrip_script::{GameScript, known_bug, run};
use serde_json::Value;

fn scripts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios")
}

/// Plays one scenario at a table with Bot1 driven. Ok, or the verdict's report.
fn at_a_table(name: &str, text: &str) -> Result<(), String> {
    let mut session = Session::scenario(text, name, Vec::new(), &[1]).map_err(|r| r.detail)?;
    let mut step = session.step();
    for _ in 0..20_000 {
        let scenario = &step["Scenario"];
        if let Some(verdict) = scenario["Verdict"].as_object() {
            return if verdict["Passed"] == Value::Bool(true) { Ok(()) } else { Err(verdict["Report"].as_str().unwrap_or("").to_string()) };
        }
        let expected = &scenario["Expected"];
        if expected.is_null() {
            return Err(format!("the step says nothing to answer and gives no verdict: {}", step["Requests"]));
        }
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).map_err(|e| format!("the expected answer does not read back: {e}"))?
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| format!("{}: {} ({:?}, line {})", r.code, r.detail, answer, expected["Line"]))?;
    }
    Err("the scenario went on too long at the table".into())
}

/// The client must tap mana sources before choosing the tapped creature returned for web-slinging (CR 601.2g-h).
#[test]
fn web_slinging_cast_rows_require_mana_before_the_tapped_creature_cost() {
    fn check_rows(value: &Value, source: &Value, saw_web: &mut bool, saw_printed: &mut bool) {
        match value {
            Value::Object(fields) => {
                if fields.get("Kind").and_then(Value::as_str) == Some("CastSpell")
                    && fields.get("Source") == Some(source) {
                    let web = fields.get("Description").and_then(Value::as_str).is_some_and(|s| s.contains("Web-slinging"));
                    assert_eq!(fields.get("TappedAfterMana"), Some(&Value::Bool(web)));
                    if web { *saw_web = true; } else { *saw_printed = true; }
                }
                for value in fields.values() { check_rows(value, source, saw_web, saw_printed); }
            }
            Value::Array(values) => for value in values { check_rows(value, source, saw_web, saw_printed); },
            _ => {}
        }
    }
    let (mut saw_web, mut saw_printed) = (false, false);
    play_looking("spider-sense-web-slinging-returns-the-birds-tapped-for-its-own-mana", |_, views, step| {
        if let Some(card) = views[0].objects.iter().find(|o| o.name == "Spider-Sense") {
            check_rows(&step["Requests"], &serde_json::to_value(card.handle).unwrap(), &mut saw_web, &mut saw_printed);
        }
        false
    });
    assert!(saw_web, "no web-slinging cast row was inspected");
    assert!(saw_printed, "no printed-cost cast row was inspected");
}

/// Equal energy payments remain distinguishable because the graveyard permissions have different consequences.
#[test]
fn energy_cast_rows_name_the_selected_graveyard_permission() {
    let name = "primal-prayers-chooses-between-granted-graveyard-permissions";
    let mut saw_both = false;
    fn descriptions(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Object(fields) => {
                if let Some(text) = fields.get("Description").and_then(Value::as_str) {
                    out.push(text.to_owned());
                }
                for value in fields.values() { descriptions(value, out); }
            }
            Value::Array(values) => for value in values { descriptions(value, out); },
            _ => {}
        }
    }
    play_looking(name, |_, _, step| {
        let mut rows = Vec::new();
        descriptions(&step["Requests"], &mut rows);
        let has_permission = |name: &str| rows.iter().any(|text|
            text.contains("Pay {E}") && text.contains(&format!("Permission: {name}")));
        saw_both |= has_permission("Lurrus of the Dream-Den") && has_permission("Serra Paragon");
        false
    });
    assert!(saw_both, "no prompt showed both energy routes with their permission sources");
}

/// Hidden information (CR 406.3, 708.2): Headliner Scarlett exiles Bot2's top card, Firdoch Core, face down. What Bot1's
/// view says of that card - at the table, in the views as they are sent - carries nothing of what it is: no name, and not
/// that it is every creature type (Changeling), which would say it.
#[test]
fn a_face_down_changeling_in_exile_tells_the_opponent_nothing_of_what_it_is() {
    let name = "headliner-scarlett-face-down-firdoch-core-shows-an-opponent-no-types";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let mut seen = 0;
    for _ in 0..20_000 {
        let view = session.view(1).map_err(|r| r.detail).unwrap();
        for object in view.objects.iter().filter(|o| o.face_down) {
            seen += 1;
            assert!(!object.every_creature_type, "a face-down card says it is every creature type: {:?}", object.handle);
            assert!(object.type_line_subtypes.is_empty() && object.subtypes.is_empty(), "a face-down card names types");
            assert!(object.granted_abilities.is_empty() && object.lost_abilities.is_empty() && object.name.is_empty());
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(seen > 0, "the scenario never showed Bot1 a face-down card");
}

/// A restriction on casting is shown to the player it binds (CR 101.2): while Avatar's Wrath bars Bot2 from casting outside
/// its hand, Bot2's view says so beside its life total and Bot1's says it of Bot2, and once Bot1's next turn has begun
/// neither says it any more.
#[test]
fn a_bar_on_casting_outside_the_hand_is_shown_to_both_players_while_it_lasts() {
    let name = "avatars-wrath-bars-its-opponent-casting-from-exile-and-graveyard-until-its-casters-next-turn";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut shown, mut gone_after) = (0, false);
    for _ in 0..20_000 {
        let own = session.view(2).map_err(|r| r.detail).unwrap();
        let other = session.view(1).map_err(|r| r.detail).unwrap();
        let facts = |view: &cantrip_host::protocol::PlayerView| view.players[1].pending_effects.clone();
        let (own_facts, other_facts) = (facts(&own), facts(&other));
        let bot1 = own.players[0].name.clone();
        let barred = own_facts.contains(&format!("You can't cast spells from anywhere other than your hand until {bot1}'s next turn."));
        assert_eq!(
            barred,
            other_facts.contains(&"They can't cast spells from anywhere other than their hand until your next turn.".to_string()),
            "the two players are told different things: {own_facts:?} / {other_facts:?}"
        );
        if barred {
            shown += 1;
            assert!(!gone_after, "the restriction came back");
        } else if shown > 0 {
            gone_after = true;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(shown > 0, "Bot2 was never told it can't cast outside its hand");
    assert!(gone_after, "the restriction was still shown after Bot1's next turn began");
}

/// "Players can't gain life" is shown to both players while it applies (CR 119.7): while Bot1's Sunspine Lynx is on the
/// battlefield, each player's view says it of them ("You can't gain life.") and of the other ("They can't gain life."), and once
/// Swords to Plowshares has exiled the Lynx neither says it any more.
#[test]
fn players_cant_gain_life_is_shown_to_both_players_while_the_static_ability_applies() {
    let name = "sunspine-lynx-stops-life-gain-only-while-it-is-on-the-battlefield";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut shown, mut gone_after) = (0, false);
    for _ in 0..20_000 {
        let views = [session.view(1).map_err(|r| r.detail).unwrap(), session.view(2).map_err(|r| r.detail).unwrap()];
        let says = |viewer: usize, about: usize, text: &str| views[viewer].players[about].pending_effects.iter().any(|f| f == text);
        let applies = says(0, 0, "You can't gain life.");
        for (viewer, about) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            let text = if viewer == about { "You can't gain life." } else { "They can't gain life." };
            assert_eq!(says(viewer, about, text), applies, "seat {} is told something else of seat {}", viewer + 1, about + 1);
        }
        if applies {
            shown += 1;
            assert!(!gone_after, "the restriction came back");
        } else if shown > 0 {
            gone_after = true;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(shown > 0, "no player was ever told they can't gain life");
    assert!(gone_after, "the restriction was still shown after the Lynx left");
}

/// Plays the scenario `name` at a table and judges what each seat is told, beside Bot1's life total, of a delayed triggered
/// ability that waits for its event (CR 603.7b) whose words contain `words`: nothing before it is set up, then Bot1's own view
/// names its source and gives the words, Bot2's says it is Bot1's ("They have ..."), and once it has lapsed neither does.
fn judge_waiting_delayed_ability(name: &str, words: &str, source: &str) {
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut shown, mut gone_after, mut first) = (0, false, true);
    for _ in 0..20_000 {
        let mut facts = |seat: i32| session.view(seat).map_err(|r| r.detail).unwrap().players[0].pending_effects.clone();
        let (own, other) = (facts(1), facts(2));
        let own_line = own.iter().find(|f| f.contains(words));
        let other_line = other.iter().find(|f| f.contains(words));
        assert_eq!(own_line.is_some(), other_line.is_some(), "the two seats are told different things: {own:?} / {other:?}");
        if first {
            assert!(own_line.is_none(), "the ability was shown before anything set it up: {own:?}");
            first = false;
        }
        if let (Some(mine), Some(theirs)) = (own_line, other_line) {
            assert!(mine.starts_with(&format!("{source}: ")), "Bot1 is not told where it came from: {mine}");
            assert!(theirs.starts_with(&format!("They have {source}'s delayed ability: ")), "Bot2 is told something else: {theirs}");
            shown += 1;
            assert!(!gone_after, "the waiting ability came back");
        } else if shown > 0 {
            gone_after = true;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(shown > 0, "the waiting ability was never shown");
    assert!(gone_after, "the waiting ability was still shown after it lapsed");
}

/// "Whenever you attack this turn" (Dalkovan Encampment's activated ability) is shown from the moment it is set up until the
/// turn ends.
#[test]
fn a_this_turn_delayed_ability_is_shown_until_the_turn_ends() {
    judge_waiting_delayed_ability(
        "dalkovan-encampment-shows-the-waiting-attack-ability-and-the-warriors-in-the-browser",
        "Whenever you attack this turn",
        "Dalkovan Encampment",
    );
}

/// An "until your next turn" delayed ability (Tamiyo, Seasoned Scholar's +2) is shown until that turn begins.
#[test]
fn an_until_your_next_turn_delayed_ability_is_shown_until_that_turn_begins() {
    judge_waiting_delayed_ability(
        "tamiyo-seasoned-scholar-plus-two-weakens-creatures-attacking-her-too",
        "Whenever a creature attacks you or a planeswalker you control",
        "Tamiyo, Seasoned Scholar",
    );
}

/// CR 305.7 at a table: under Magus of the Moon, Bot2's Azorius Chancery is a Mountain. Each seat's view of it, at the end of
/// the scenario, gives its subtypes as Mountain alone and calls the three lines its rules text printed lost: "This land enters
/// tapped." (a rule of its card, not an ability among its abilities), its enters trigger, and its "{T}: Add {W}{U}." Its
/// name stays.
#[test]
fn a_nonbasic_land_magus_of_the_moon_makes_a_mountain_is_shown_with_its_printed_lines_lost() {
    let name = "magus-of-the-moon-lets-azorius-chancery-enter-untapped-without-its-trigger";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    for _ in 0..20_000 {
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    for seat in [1, 2] {
        let view = session.view(seat).map_err(|r| r.detail).unwrap();
        let chancery = view.objects.iter().find(|o| o.name == "Azorius Chancery").expect("the Chancery is shown");
        assert_eq!(chancery.subtypes, vec!["Mountain".to_string()], "seat {seat}");
        for line in ["This land enters tapped.", "When this land enters, return a land you control to its owner's hand.", "{T}: Add {W}{U}."] {
            assert!(chancery.lost_abilities.iter().any(|l| l == line), "seat {seat} is not told {line:?} is lost: {:?}", chancery.lost_abilities);
        }
        assert!(chancery.granted_abilities.is_empty(), "seat {seat}: {:?}", chancery.granted_abilities.iter().map(|a| &a.text).collect::<Vec<_>>());
    }
}

/// CR 605.1a, 305.6 at a table: under Blood Sun ("All lands lose all abilities except mana abilities") Taiga keeps the mana
/// abilities of its land types, so the reminder text that is its whole text box is not shown as lost.
#[test]
fn a_land_under_blood_sun_is_not_shown_losing_its_land_types_mana_abilities() {
    let name = "blood-sun-leaves-taiga-the-mana-abilities-of-its-land-types";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    for _ in 0..20_000 {
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    for seat in [1, 2] {
        let view = session.view(seat).map_err(|r| r.detail).unwrap();
        let taiga = view.objects.iter().find(|o| o.name == "Taiga").expect("Taiga is shown");
        assert!(taiga.lost_abilities.is_empty(), "seat {seat}: {:?}", taiga.lost_abilities);
    }
}

#[test]
fn every_scenario_passes_at_a_table_with_bot1_driven() {
    let mut entries: Vec<PathBuf> =
        std::fs::read_dir(scripts()).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "cantrip")).collect();
    entries.sort();
    let mut failed = Vec::new();
    let mut played = 0;
    for path in entries {
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let Ok(script) = GameScript::parse(&text, &name) else { continue };
        if known_bug(&text).is_some() || script.cards().into_iter().any(|c| cantrip_engine::card(&c).is_none()) {
            continue;
        }
        if run(&script).is_err() {
            continue;
        }
        played += 1;
        if let Err(report) = at_a_table(&name, &text) {
            failed.push(format!("{name}: {}", report.lines().take(3).collect::<Vec<_>>().join(" | ")));
        }
    }
    assert!(failed.is_empty(), "{} of {played} scenarios fail at a table:\n{}", failed.len(), failed.join("\n"));
}

/// A spell being announced and paid for (CR 601.2a-h) is the caster's alone until it is cast (CR 601.2i), as the events
/// already have it: Bot2's view, as each step sends it, shows no Lightning Bolt on the stack and no Mountain tapped while
/// Bot1 is choosing its target and paying - a cast that is taken back is never seen - and shows the Bolt once it is cast.
#[test]
fn a_spell_being_announced_is_not_shown_to_the_opponent_until_it_is_cast() {
    let text = "[Bot1 Deck]\nMountain\nLightning Bolt\nMountain x18\n\n[Bot2 Deck]\nIsland x20\n\n[Actions]\nBot1.Hand.Mountain.Play\nBot1.Hand.Lightning Bolt.Cast(Bot2)\nBot1.Battlefield.Mountain.Activate\n\n[Expect]\nBot2.Life = 17\n";
    let mut session = Session::scenario(text, "announced-bolt", Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let stack = |view: &Value| -> i64 {
        view["Zones"].as_array().unwrap().iter().find(|z| z["Kind"] == "Stack").map_or(0, |z| z["Count"].as_i64().unwrap())
    };
    let tapped = |view: &Value| view["Objects"].as_array().unwrap().iter().filter(|o| o["Tapped"] == Value::Bool(true)).count();
    let (mut announcing, mut cast) = (0, 0);
    for _ in 0..20_000 {
        let (own, other) = (&step["Views"]["1"], &step["Views"]["2"]);
        if stack(own) > 0 && stack(other) == 0 {
            announcing += 1;
            assert_eq!(tapped(other), 0, "Bot2 saw a Mountain tapped to pay for a spell still being cast");
            assert_eq!(other["Players"][0]["HandCount"], 6, "Bot2 saw the Bolt leave Bot1's hand");
        }
        if stack(other) > 0 {
            cast += 1;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true), "{}", scenario["Verdict"]["Report"]);
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = serde_json::from_value(expected["Answer"].clone()).unwrap();
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(announcing > 0, "Bot1 was never asked anything while the Bolt was being announced");
    assert!(cast > 0, "Bot2 never saw the Bolt on the stack once it was cast");
}

/// Outrageous Robbery (CR 406.3): "You may look at" the cards it exiles face down. Bot1, who looked, is told what the card is
/// (`face_down_name`, `known`); Bot2, whose library it came out of and who never looked at it, is told nothing of it - no
/// name, not known - though it is a card of Bot2's in Bot2's own exile.
#[test]
fn a_card_robbery_exiles_face_down_is_named_to_the_caster_and_not_to_its_owner() {
    let name = "outrageous-robbery-in-the-browser-offers-the-caster-the-face-down-card-in-the-opponents-exile";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut caster_saw, mut owner_saw) = (0, 0);
    for _ in 0..20_000 {
        let caster = session.view(1).map_err(|r| r.detail).unwrap();
        let owner = session.view(2).map_err(|r| r.detail).unwrap();
        for object in caster.objects.iter().filter(|o| o.face_down) {
            assert_eq!(object.face_down_name, "Lightning Helix", "the caster is not told what it looked at");
            assert!(object.known && object.name.is_empty());
            caster_saw += 1;
        }
        for object in owner.objects.iter().filter(|o| o.face_down) {
            assert!(object.face_down_name.is_empty() && !object.known && object.name.is_empty(), "the owner is told what the card is: {:?}", object.face_down_name);
            owner_saw += 1;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(caster_saw > 0 && owner_saw > 0, "the scenario never showed both seats the face-down card");
}

/// Plays the scenario `name` at a table, Bot1 driven by the script, calling `look` with both seats' views and the step before
/// each answer; stops early when `look` says so. The last step.
fn play_looking(name: &str, mut look: impl FnMut(&mut Session, &[cantrip_host::protocol::PlayerView; 2], &Value) -> bool) -> Value {
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    for _ in 0..20_000 {
        let views = [session.view(1).map_err(|r| r.detail).unwrap(), session.view(2).map_err(|r| r.detail).unwrap()];
        if look(&mut session, &views, &step) {
            return step;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true), "{}", scenario["Verdict"]["Report"]);
            return step;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    panic!("the scenario went on too long at the table");
}

/// Whether `viewer`'s view (0 or 1) says `text` beside the life total of `about`.
fn says(views: &[cantrip_host::protocol::PlayerView; 2], viewer: usize, about: usize, text: &str) -> bool {
    views[viewer].players[about].pending_effects.iter().any(|f| f == text)
}

/// A game drawn by a loop of mandatory actions (CR 104.4b) says so to every seat: the three-Oblivion-Ring loop ends with a
/// `GameEndedEvent` told to both players, a draw, and its reason the loop - what the client's game-over panel and log say.
#[test]
fn a_game_drawn_by_a_mandatory_loop_tells_every_seat_why() {
    let mut ended: Vec<(String, Value)> = Vec::new();
    let last = play_looking("three-oblivion-rings-with-no-other-nonland-permanent-loop-forever-and-the-game-is-a-draw", |_, _, step| {
        for (seat, list) in step["Events"].as_object().unwrap() {
            for event in list.as_array().unwrap().iter().filter(|e| e["$type"] == "GameEndedEvent") {
                ended.push((seat.clone(), event.clone()));
            }
        }
        false
    });
    assert_eq!(last["Outcome"], "Draw");
    let mut seats: Vec<&str> = ended.iter().map(|(s, _)| s.as_str()).collect();
    seats.sort_unstable();
    assert_eq!(seats, ["1", "2"], "each seat is told once: {ended:?}");
    for (_, event) in &ended {
        assert_eq!(event["Outcome"], "Draw");
        assert_eq!(event["DrawReason"], "MandatoryLoop", "{event}");
    }
}

/// Platinum Angel's state is shown to both players (CR 104.2b, 104.3): while it is out, Bot1 is told "You can't lose the game."
/// and Bot2 that Bot1 can't ("They can't lose the game."), and Bot2 that it can't win, which Bot1 is told of Bot2. At 0 life
/// a concession still loses (CR 104.3a): the game is over, Bot2 the winner.
#[test]
fn platinum_angel_is_shown_and_a_concession_still_loses() {
    let mut shown = false;
    let mut conceded = false;
    play_looking("platinum-angel-keeps-its-controller-in-at-zero-life-until-it-is-destroyed", |session, views, _| {
        let applies = says(views, 0, 0, "You can't lose the game.");
        assert_eq!(says(views, 1, 0, "They can't lose the game."), applies, "the seats are told different things of Bot1");
        assert_eq!(says(views, 1, 1, "You can't win the game."), applies, "Bot2 is not told it can't win");
        assert_eq!(says(views, 0, 1, "They can't win the game."), applies, "Bot1 is not told Bot2 can't win");
        shown |= applies;
        if applies && views[0].players[0].life == 0 {
            let step = session.concede(1).map_err(|r| r.detail).unwrap();
            assert_eq!(step["Outcome"], "TeamWon");
            assert_eq!(step["WinningTeam"], 2);
            conceded = true;
            return true;
        }
        false
    });
    assert!(shown, "nobody was told of the Angel");
    assert!(conceded, "Bot1 was never at 0 life with the Angel out");
}

/// Phyrexian Unlife's state is shown to both players (CR 104.3b, 702.90b): "You don't lose the game for having 0 or less life."
/// while it is out, and, once Bot1 is at 0 or less life, "Damage is dealt to you as though its source had infect." - each told
/// to Bot2 of Bot1 in its own words. Melira's "You can't get poison counters." likewise.
#[test]
fn phyrexian_unlife_and_melira_are_shown_to_both_players() {
    let (mut unlife, mut infect, mut melira) = (false, false, false);
    play_looking("melira-with-phyrexian-unlife-damage-dealt-as-poison-does-nothing", |_, views, _| {
        let at_most_zero = views[0].players[0].life <= 0;
        let pairs = [
            ("You don't lose the game for having 0 or less life.", "They don't lose the game for having 0 or less life.", &mut unlife),
            ("Damage is dealt to you as though its source had infect.", "Damage is dealt to them as though its source had infect.", &mut infect),
            ("You can't get poison counters.", "They can't get poison counters.", &mut melira),
        ];
        let unlife_out = says(views, 0, 0, pairs[0].0);
        for (own, other, seen) in pairs {
            let applies = says(views, 0, 0, own);
            assert_eq!(says(views, 1, 0, other), applies, "the seats are told different things: {own}");
            *seen |= applies;
        }
        assert_eq!(says(views, 0, 0, "Damage is dealt to you as though its source had infect."), at_most_zero && unlife_out);
        false
    });
    assert!(unlife && infect && melira, "not everything was shown: {unlife} {infect} {melira}");
}

/// The granted lines of the permanent named `name` in `view`, if it is shown.
fn granted_lines(view: &cantrip_host::protocol::PlayerView, name: &str) -> Option<Vec<String>> {
    view.objects.iter().find(|o| o.name == name).map(|o| o.granted_abilities.iter().map(|a| a.text.clone()).collect())
}

/// Protection a permanent was given (CR 702.16) is shown on it to both seats, worded as cards print it: Akroma's Memorial's
/// "protection from black and from red" on Bot1's Hired Blade, and Mother of Runes's "protection from red". Progenitus's own
/// protection from everything is its printed line and is not shown again as granted.
#[test]
fn protection_a_permanent_was_given_is_shown_and_a_printed_one_is_not_repeated() {
    let mut memorial = [false, false];
    play_looking("akromas-memorial-protection-from-red-drops-a-red-aura-and-equipment-and-turns-a-bolt-away", |_, views, _| {
        for (seat, view) in views.iter().enumerate() {
            if granted_lines(view, "Hired Blade").is_some_and(|lines| lines.iter().any(|l| l == "Protection from black and from red")) {
                memorial[seat] = true;
            }
        }
        false
    });
    assert_eq!(memorial, [true, true], "both seats are told the Blade has protection from black and from red");

    let mut mother = false;
    play_looking("mother-of-runes-after-blocks-leaves-the-blocker-blocking-and-prevents-its-damage", |_, views, _| {
        mother |= granted_lines(&views[1], "Hired Blade").is_some_and(|lines| lines.iter().any(|l| l == "Protection from red"));
        false
    });
    assert!(mother, "Bot2 is told the Blade Mother of Runes protected has protection from red");

    // Mother of Runes on herself: her own text says "protection from", and the protection an effect gave her is still shown.
    let mut herself = false;
    play_looking("mother-of-runes-in-response-makes-a-lightning-bolt-fizzle", |_, views, _| {
        herself |= granted_lines(&views[0], "Mother of Runes").is_some_and(|lines| lines.iter().any(|l| l == "Protection from red"));
        false
    });
    assert!(herself, "Bot1 is told Mother of Runes gave herself protection from red");

    let mut progenitus_seen = false;
    play_looking("progenitus-cannot-be-targeted-blocked-enchanted-or-damaged", |_, views, _| {
        for view in views {
            if let Some(lines) = granted_lines(view, "Progenitus") {
                progenitus_seen = true;
                assert!(!lines.iter().any(|l| l.starts_with("Protection")), "Progenitus's printed protection is shown as granted: {lines:?}");
            }
        }
        false
    });
    assert!(progenitus_seen, "the scenario never showed Progenitus");
}

/// A player's protection from a player (CR 702.16k) is shown to both players beside the protected player, as control is now:
/// while Bot1 controls Absolute Virtue, Bot1 is told "You have protection from Bot2." and Bot2 that Bot1 has it; once Bot2's
/// Act of Treason has taken the Virtue, Bot2 is told it has protection from Bot1, and Bot1 is told nothing of its own.
#[test]
fn protection_from_a_player_is_shown_to_both_players_and_follows_control() {
    let (mut bot1_protected, mut bot2_protected) = (false, false);
    play_looking("absolute-virtue-stolen-protects-its-new-controller-and-not-its-owner", |_, views, _| {
        let first = says(views, 0, 0, "You have protection from Bot2.");
        assert_eq!(says(views, 1, 0, "They have protection from Bot2."), first, "the seats are told different things of Bot1");
        let second = says(views, 1, 1, "You have protection from Bot1.");
        assert_eq!(says(views, 0, 1, "They have protection from Bot1."), second, "the seats are told different things of Bot2");
        assert!(!(first && second), "both players were shown protection at once");
        assert!(!(bot2_protected && first), "Bot1's protection came back while Bot2 controlled the Virtue");
        bot1_protected |= first;
        bot2_protected |= second;
        false
    });
    assert!(bot1_protected && bot2_protected, "not both were shown: {bot1_protected} {bot2_protected}");
}
