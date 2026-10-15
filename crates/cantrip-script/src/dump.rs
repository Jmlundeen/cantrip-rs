//! The position as a flat map of keys to strings, as `src/Cantrip.Scripting/StateDump.cs` writes it, and the
//! comparison a scenario's expectations are checked by.

use std::collections::{BTreeMap, BTreeSet};

use cantrip_engine::*;

use crate::parse::Expectation;

#[derive(Debug, Clone, Default)]
pub struct StateDump {
    pub values: BTreeMap<String, String>,
    /// Keys whose lists compare as sets.
    unordered: BTreeSet<String>,
}

pub fn bot(p: PlayerId) -> String {
    format!("Bot{}", p.0 + 1)
}

pub fn step_short(step: Step) -> &'static str {
    match step {
        Step::Untap => "Untap",
        Step::Upkeep => "Upkeep",
        Step::Draw => "Draw",
        Step::PrecombatMain => "Main1",
        Step::BeginCombat => "BeginCombat",
        Step::DeclareAttackers => "Attackers",
        Step::DeclareBlockers => "Blockers",
        Step::CombatDamage => "Damage",
        Step::EndOfCombat => "EndCombat",
        Step::PostcombatMain => "Main2",
        Step::End => "End",
        Step::Cleanup => "Cleanup",
    }
}

/// The order a type line prints card types in, which is the order the C# engine lists them: "Kindred Creature",
/// "Artifact Creature", "Enchantment Creature", "Land Creature". A card's own types come first, in this order, and
/// the types effects added to it after them.
const TYPE_ORDER: [CardType; 9] = [
    CardType::Kindred,
    CardType::Artifact,
    CardType::Enchantment,
    CardType::Land,
    CardType::Planeswalker,
    CardType::Battle,
    CardType::Creature,
    CardType::Instant,
    CardType::Sorcery,
];

fn keyword_name(k: u8) -> &'static str {
    match k {
        0 => "Flying",
        1 => "Haste",
        2 => "Vigilance",
        3 => "Trample",
        4 => "FirstStrike",
        5 => "DoubleStrike",
        6 => "Deathtouch",
        7 => "Lifelink",
        8 => "Reach",
        9 => "Defender",
        10 => "Flash",
        11 => "Hexproof",
        12 => "Indestructible",
        13 => "Menace",
        14 => "Infect",
        15 => "Shadow",
        16 => "Shroud",
        17 => "CantBeCountered",
        18 => "CantBlock",
        19 => "CanBlockOnlyCreaturesWithFlying",
        20 => "DoesNotUntap",
        21 => "CrewsAsThoughPowerTwoGreater",
        22 => "CantBeBlocked",
        29 => "CantBeAttacked",
        30 => "CantBeBlockedByPowerTwoOrLess",
        31 => "CantBeBlockedExceptByThreeOrMore",
        23 => "CantSpendManaToCast",
        24 => "CastFromYourGraveyard",
        25 => "Retrace",
        26 => "Daybound",
        27 => "Nightbound",
        28 => "SplitSecond",
        _ => "?",
    }
}

fn counter_token(kind: CounterKind) -> &'static str {
    kind.token()
}

pub fn mana_text(pool: &ManaPool) -> String {
    let mut s = String::new();
    for mana in Mana::ALL {
        for _ in 0..pool.get(mana) {
            s.push('{');
            s.push(mana.symbol());
            s.push('}');
        }
    }
    s
}

impl StateDump {
    fn set(&mut self, key: impl Into<String>, value: impl ToString) {
        self.values.insert(key.into(), value.to_string());
    }

    fn list(&mut self, key: &str, mut items: Vec<String>, ordered: bool) {
        if !ordered {
            items.sort();
            self.unordered.insert(key.to_string());
        }
        self.values.insert(format!("{key}.Count"), items.len().to_string());
        self.values.insert(key.to_string(), items.join("; "));
    }

    pub fn of(game: &Game) -> StateDump {
        let s = &game.state;
        let mut d = StateDump::default();
        d.set("Turn", s.turn.number);
        d.set("Step", step_short(s.turn.step()));
        d.set("ActivePlayer", bot(s.turn.active));
        d.set("PriorityPlayer", bot(game.pending().map(|r| r.player).unwrap_or(s.turn.priority)));
        d.set(
            "Result",
            match s.outcome {
                Outcome::InProgress => "InProgress".to_string(),
                Outcome::Draw => "Draw".to_string(),
                Outcome::Won(p) => format!("{} won", bot(p)),
            },
        );

        let stack: Vec<String> = s.stack.iter().rev().map(|item| game.name(item.object).to_string()).collect();
        d.list("Stack", stack, true);

        let mut exiled: Vec<ObjectId> = s.seats().flat_map(|p| s.zone(Zone::exile(p)).to_vec()).collect();
        exiled.sort();
        // A card exiled face down has no name in exile (CR 406.3): `Exile` lists it as "(face down)", and `Exile.FaceDown`, the
        // dump's own omniscient view, lists what those cards are.
        let face_down = |id: &ObjectId| s.obj(*id).face_down;
        d.list("Exile", exiled.iter().map(|id| if face_down(id) { "(face down)".to_string() } else { game.name(*id).to_string() }).collect(), false);
        let hidden: Vec<String> = exiled.iter().filter(|id| face_down(id)).map(|&id| game.name(id).to_string()).collect();
        if !hidden.is_empty() {
            d.list("Exile.FaceDown", hidden, false);
        }

        let attackers = s
            .attackers
            .iter()
            .map(|a| {
                let at = match a.target {
                    Target::Player(p) => bot(p),
                    Target::Object(id) => game.name(id).to_string(),
                    Target::None | Target::Ability(_) => "nothing".into(),
                };
                format!("{} > {}", game.name(a.id), at)
            })
            .collect();
        d.list("Combat.Attackers", attackers, false);
        let blocks = s.blocks.iter().map(|b| format!("{} > {}", game.name(b.blocker), game.name(b.attacker))).collect();
        d.list("Combat.Blocks", blocks, false);

        // Only once there is one (CR 725.1, 726.1): most games never have a monarch or the initiative.
        if let Some(p) = s.seats().find(|&p| s.player(p).monarch) {
            d.set("Monarch", bot(p));
        }
        if let Some(p) = s.seats().find(|&p| s.player(p).initiative) {
            d.set("Initiative", bot(p));
        }
        // Day and night (CR 731.1), the game's own designation: absent while it is neither.
        match s.turn.day_night {
            DayNight::Neither => {}
            DayNight::Day => d.set("DayNight", "Day"),
            DayNight::Night => d.set("DayNight", "Night"),
        }
        for p in s.seats() {
            let b = bot(p);
            let player = s.player(p);
            // The cards the player has looked at, in order (`Event::Looked`), once they have looked at any.
            let looked: Vec<String> = game
                .history()
                .select(EventQuery::kind(EventMask::LOOKED).player(p))
                .filter_map(|r| match r.event {
                    Event::Looked { card, .. } => Some(game.name(card).to_string()),
                    _ => None,
                })
                .collect();
            if !looked.is_empty() {
                d.list(&format!("{b}.Looked"), looked, true);
            }
            if player.room != 0
                && let Some(dungeon) = game.cards.undercity
            {
                let label = label_text(&Label::Card(dungeon, player.room - 1));
                d.set(format!("{b}.Dungeon"), format!("Undercity: {}", label.split(" \u{2014} ").next().unwrap_or(label)));
            }
            if player.dungeons_completed > 0 {
                d.set(format!("{b}.DungeonsCompleted"), player.dungeons_completed);
            }
            d.set(format!("{b}.Life"), player.life);
            d.set(format!("{b}.Poison"), player.poison);
            if player.lost {
                d.set(format!("{b}.Status"), "Lost");
            }
            d.set(format!("{b}.Energy"), player.energy);
            if player.experience > 0 {
                d.set(format!("{b}.Experience"), player.experience);
            }
            d.set(format!("{b}.LandsPlayed"), player.lands_played);
            let lasting = player.lasting_mana();
            let mut mana = if lasting.is_empty() {
                mana_text(&player.pool)
            } else {
                format!("{} lasting {}", mana_text(&player.pool), mana_text(&lasting))
            };
            // Restricted mana apart, each kind with its restriction: "{R}{R} (GainsHaste)".
            for restriction in ManaRestriction::ALL {
                for m in Mana::ALL {
                    let n = player.restricted[restriction as usize].get(m);
                    if n > 0 {
                        let mut one = ManaPool::default();
                        one.0[m as usize] = n;
                        mana = format!("{mana} {} ({})", mana_text(&one), restriction.name()).trim().to_string();
                    }
                }
            }
            d.set(format!("{b}.Mana"), mana);
            let names = |ids: &[ObjectId]| ids.iter().map(|&id| game.name(id).to_string()).collect::<Vec<_>>();
            d.list(&format!("{b}.Hand"), names(s.zone(Zone::hand(p))), false);
            d.list(&format!("{b}.Library"), names(s.zone(Zone::library(p))), true);
            let mut graveyard = names(s.zone(Zone::graveyard(p)));
            graveyard.reverse();
            d.list(&format!("{b}.Graveyard"), graveyard, true);

            // CR 114: the emblems a player owns, from the one shared command zone.
            let emblems = s.zone(Zone::COMMAND).iter().filter(|&&id| s.obj(id).owner == p).map(|&id| game.name(id).to_string()).collect();
            d.list(&format!("{b}.Emblems"), emblems, false);

            let permanents: Vec<ObjectId> = s.battlefield().iter().copied().filter(|&id| s.obj(id).controller == p).collect();
            d.list(&format!("{b}.Battlefield"), names(&permanents), false);
            for &id in &permanents {
                let name = game.name(id);
                let same: Vec<ObjectId> = permanents.iter().copied().filter(|&o| game.name(o) == name).collect();
                let label = if same.len() == 1 {
                    name.to_string()
                } else {
                    format!("{name}#{}", same.iter().position(|&o| o == id).unwrap() + 1)
                };
                let key = format!("{b}.Battlefield.{label}");
                let obj = s.obj(id);
                let c = game.chars(id);
                d.set(format!("{key}.Tapped"), obj.tapped);
                // Face down (CR 708): the card it is turned up as (the dump sees everything).
                if obj.face_down {
                    d.set(format!("{key}.FaceDown"), true);
                    d.set(format!("{key}.FaceDownName"), game.cards.get(obj.card).name);
                }
                if obj.prepared {
                    d.set(format!("{key}.Prepared"), true);
                }
                // What an Aura or Equipment is attached to (CR 301.5a, 303.4), by name, while it is attached to anything.
                if let Some(host) = obj.attached_to
                    && s.obj(host).became.is_none()
                {
                    d.set(format!("{key}.AttachedTo"), game.name(host));
                }
                if obj.marks.monstrous {
                    d.set(format!("{key}.Monstrous"), true);
                }
                if game.is_saddled(id) {
                    d.set(format!("{key}.Saddled"), true);
                }
                if obj.marks.exerted_by.is_some() {
                    d.set(format!("{key}.Exerted"), true);
                }
                // The players who goaded it (CR 701.15a), while any goad lasts.
                let mut goaders: Vec<usize> =
                    s.effects.iter().filter(|e| e.affected == id && e.kind == EffectKind::Goaded).map(|e| e.controller.0 as usize + 1).collect();
                if !goaders.is_empty() {
                    goaders.sort_unstable();
                    goaders.dedup();
                    let names: Vec<String> = goaders.iter().map(|n| format!("Bot{n}")).collect();
                    d.set(format!("{key}.GoadedBy"), names.join("; "));
                }
                if let Some(chosen) = obj.marks.chosen_player {
                    d.set(format!("{key}.ChosenPlayer"), chosen.0 + 1);
                }
                if let Some(odd) = obj.marks.chosen_odd {
                    d.set(format!("{key}.ChosenParity"), if odd { "odd" } else { "even" });
                }
                // The modes its modal abilities have chosen, from the history (`Event::ModesChosen`), numbered from 1.
                let chosen = game.history().all().iter().fold(0u8, |acc, r| match r.event {
                    Event::ModesChosen { source, modes, .. } if source == id => acc | modes,
                    _ => acc,
                });
                if chosen != 0 {
                    let taken: Vec<String> = (0..8).filter(|&m| chosen & (1 << m) != 0).map(|m| (m + 1).to_string()).collect();
                    d.set(format!("{key}.ChosenModes"), taken.join(", "));
                }
                if let Some(chosen) = obj.marks.chosen_card {
                    d.set(format!("{key}.ChosenName"), game.cards.get(chosen).name);
                }
                if obj.class_level > 0 {
                    d.set(format!("{key}.ClassLevel"), obj.class_level);
                }
                // The C# engine lists a card's own types first and the types effects added after them, in the order
                // they were added: "Creature; Artifact" for a Metamorph copying a creature, "Artifact; Creature" for a
                // Sol Ring animated by an emblem.
                let own = game.cards.get(c.card).types;
                let types: Vec<String> = TYPE_ORDER
                    .iter()
                    .filter(|&&t| c.is(t) && own.has(t))
                    .chain(TYPE_ORDER.iter().filter(|&&t| c.is(t) && !own.has(t)))
                    .map(|t| t.name().to_string())
                    .collect();
                d.list(&format!("{key}.Types"), types, true);
                let mut subtypes: Vec<String> = c.subtypes.iter().map(|t| t.name().to_string()).collect();
                if !subtypes.is_empty() {
                    subtypes.sort();
                    d.list(&format!("{key}.Subtypes"), subtypes, true);
                }
                let mut colors: Vec<String> = c.colors.iter().map(|c| c.name().to_string()).collect();
                if !colors.is_empty() {
                    colors.sort();
                    d.list(&format!("{key}.Colors"), colors, true);
                }
                let mut keywords: Vec<String> =
                    (0..32u8).filter(|&k| c.keywords.0 & (1 << k) != 0).map(|k| keyword_name(k).to_string()).collect();
                if !keywords.is_empty() {
                    keywords.sort();
                    d.list(&format!("{key}.Keywords"), keywords, true);
                }
                if c.is(CardType::Creature) {
                    d.set(format!("{key}.Power"), c.power.unwrap_or(0));
                    d.set(format!("{key}.Toughness"), c.toughness.unwrap_or(0));
                    d.set(format!("{key}.Damage"), obj.damage);
                    d.set(format!("{key}.SummoningSick"), summoning_sick(game.cards, s, id));
                }
                // A colour chosen as it entered and kept on it (CR 614.12a).
                if let Some(color) = obj.chosen_color {
                    d.set(format!("{key}.ChosenColor"), color.name());
                }
                // A creature type chosen as it entered and kept on it (CR 614.12a).
                if let Some(subtype) = obj.chosen_subtype {
                    d.set(format!("{key}.ChosenSubtype"), subtype.name());
                }
                for kind in CounterKind::ALL {
                    let n = obj.counters(kind);
                    if n != 0 {
                        d.set(format!("{key}.Counters.{}", counter_token(kind)), n);
                    }
                }
            }
        }
        d
    }

    /// What does not hold, one line each.
    pub fn mismatches(&self, expectations: &[Expectation]) -> Vec<String> {
        let mut out = Vec::new();
        for e in expectations {
            if e.value == "(absent)" {
                if let Some(actual) = self.values.get(&e.key) {
                    out.push(format!("line {}: {} expected to be absent, was \"{actual}\"", e.line, e.key));
                } else if let Some(owner) = owner(&e.key)
                    && !self.values.keys().any(|k| k.starts_with(&owner))
                {
                    out.push(format!("line {}: {} cannot be absent: the dump has no {} to look on", e.line, e.key, owner));
                }
                continue;
            }
            let Some(actual) = self.values.get(&e.key) else {
                let near: Vec<&String> = self.values.keys().filter(|k| close(k, &e.key)).take(6).collect();
                out.push(format!("line {}: {} is not a key the dump has (keys like it: {near:?})", e.line, e.key));
                continue;
            };
            if !same(self.unordered.contains(&e.key), &e.value, actual) {
                out.push(format!("line {}: {} expected \"{}\", was \"{actual}\"", e.line, e.key, e.value));
            }
        }
        out
    }

    pub fn render(&self) -> String {
        self.values.iter().map(|(k, v)| format!("  {k} = {v}\n")).collect()
    }
}

fn owner(key: &str) -> Option<String> {
    let dot = key.rfind('.')?;
    let mut parent = &key[..dot];
    if let Some(stripped) = parent.strip_suffix(".Counters") {
        parent = stripped;
    }
    Some(format!("{parent}."))
}

fn close(candidate: &str, key: &str) -> bool {
    let head = key.split('.').take(2).collect::<Vec<_>>().join(".");
    candidate.starts_with(&head)
}

fn items(value: &str) -> Vec<String> {
    value.split(';').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect()
}

fn same(unordered: bool, expected: &str, actual: &str) -> bool {
    let mut e = items(expected);
    let mut a = items(actual);
    if unordered {
        e.sort();
        a.sort();
    }
    e == a
}
