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
        32 => "Changeling",
        33 => "Decayed",
        34 => "CantBeBlockedByMoreThanOne",
        35 => "Islandwalk",
        36 => "Plainswalk",
        37 => "Swampwalk",
        38 => "Mountainwalk",
        39 => "Forestwalk",
        _ => "?",
    }
}

/// An object's subtypes as the dump lists them, sorted: every creature type (Changeling, CR 702.73a) is one entry,
/// `(every creature type)`, not three hundred.
fn subtype_names(c: &Chars) -> Vec<String> {
    let every = Subtype::CREATURE_TYPES.iter().all(|&t| c.subtypes.has(t));
    let mut subtypes: Vec<String> = c
        .subtypes
        .iter()
        .filter(|t| !(every && t.is_creature_type()))
        .map(|t| t.name().to_string())
        .chain(every.then(|| "(every creature type)".to_string()))
        .collect();
    subtypes.sort();
    subtypes
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
        // Why a drawn game is drawn (CR 104.4), from the history: absent for a game not drawn.
        if let Some(reason) = game.log.iter().find_map(|r| match r.event {
            Event::GameDrawn { reason } => Some(reason),
            _ => None,
        }) {
            d.set("DrawReason", format!("{reason:?}"));
        }

        let stack: Vec<String> = s.stack.iter().rev().map(|item| game.name(item.object).to_string()).collect();
        d.list("Stack", stack, true);

        // The delayed triggered abilities waiting for their moment (CR 603.7), each named for the object it came from.
        let delayed: Vec<String> = s.delayed.iter().map(|d| game.name(d.source).to_string()).collect();
        d.list("Delayed", delayed, false);

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

        // Every permanent that left the battlefield, in order, from the history, as it last was there (CR 608.2h): "(tapped)"
        // when it was tapped as it left. A tap the history records after the permanent had left is not one it left with
        // (CR 701.26a: only a permanent becomes tapped), so a cost that taps and moves its source must pay the tap first.
        let records = game.history().all();
        let left: Vec<String> = records
            .iter()
            .enumerate()
            .filter_map(|(at, r)| match r.event {
                Event::ZoneChange { old, from, .. } if from.is(ZoneKind::Battlefield) => {
                    let tapped_later = records[at + 1..].iter().any(|later| matches!(later.event, Event::Tapped { permanent } if permanent == old));
                    let tapped = s.obj(old).tapped && !tapped_later;
                    Some(if tapped { format!("{} (tapped)", game.name(old)) } else { game.name(old).to_string() })
                }
                _ => None,
            })
            .collect();
        d.list("LeftBattlefield", left, true);

        // Every prevention of damage (CR 615.6), in order, from the history (`Event::DamagePrevented`): how much, from which
        // source, to whom, and whose prevention effect it was - "3 Lightning Bolt > Black Panther, Hope Enduring by Black
        // Panther, Hope Enduring". Absent until damage has been prevented.
        let prevented: Vec<String> = records
            .iter()
            .filter_map(|r| match r.event {
                Event::DamagePrevented { source, target, amount, by, .. } => {
                    let to = match target {
                        Target::Player(p) => bot(p),
                        Target::Object(id) => game.name(id).to_string(),
                        _ => String::new(),
                    };
                    Some(format!("{amount} {} > {to} by {}", game.name(source), game.name(by)))
                }
                _ => None,
            })
            .collect();
        if !prevented.is_empty() {
            d.list("Prevented", prevented, true);
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
        // While a declaration's cost to attack is being paid (CR 508.1h-j): the total, what is still owed, and the attackers
        // it is for. Absent at any other time.
        if let Some(Request { kind: RequestKind::PayCosts { total, remaining, attackers, .. }, .. }) = game.pending()
            && !attackers.is_empty()
        {
            d.set("Combat.AttackCost", total.to_string());
            d.set("Combat.AttackCost.Remaining", remaining.to_string());
            d.list("Combat.AttackCost.Attackers", attackers.iter().map(|&a| game.name(a).to_string()).collect(), true);
        }

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
            // The designations a keyword gives for the rest of the game (CR 702.131c, 702.195b): only once had.
            if player.citys_blessing {
                d.set(format!("{b}.CitysBlessing"), true);
            }
            if player.enduring_story {
                d.set(format!("{b}.EnduringStory"), true);
            }
            // Speed (CR 702.179): absent while the player has none (CR 702.179b).
            if let Some(speed) = player.speed {
                d.set(format!("{b}.Speed"), speed);
            }
            d.set(format!("{b}.Life"), player.life);
            d.set(format!("{b}.Poison"), player.poison);
            if player.lost {
                d.set(format!("{b}.Status"), "Lost");
            }
            // How they lost (CR 104.3), from the history: absent for a player who has not, or who left another way.
            if let Some(reason) = game.log.iter().find_map(|r| match r.event {
                Event::GameLost { player, reason } if player == p => Some(reason),
                _ => None,
            }) {
                d.set(format!("{b}.LostTo"), format!("{reason:?}"));
            }
            d.set(format!("{b}.Energy"), player.energy);
            if player.experience > 0 {
                d.set(format!("{b}.Experience"), player.experience);
            }
            d.set(format!("{b}.LandsPlayed"), player.lands_played);
            // "Skips their next N turns" (CR 500.11, 614.10): the turns still to skip, one effect each.
            let skips = s.effects.iter().filter(|e| e.kind == EffectKind::SkipNextTurn(p)).count();
            if skips > 0 {
                d.set(format!("{b}.TurnsToSkip"), skips);
            }
            let lasting = player.lasting_mana();
            let mut mana = if lasting.is_empty() {
                mana_text(&player.pool)
            } else {
                format!("{} lasting {}", mana_text(&player.pool), mana_text(&lasting))
            };
            // Restricted mana apart, each kind with its restriction: "{R}{R} (GainsHaste)".
            // In `ManaRestriction::NAMED`'s order, those of subtypes after in the order they came.
            let mut restricted: Vec<(ManaRestriction, ManaPool)> = s.restricted_of(p).collect();
            restricted.sort_by_key(|(r, _)| ManaRestriction::NAMED.iter().position(|(n, _)| n == r).unwrap_or(usize::MAX));
            for &(restriction, pool) in &restricted {
                for m in Mana::ALL {
                    let n = pool.get(m);
                    if n > 0 {
                        let mut one = ManaPool::default();
                        one.0[m as usize] = n;
                        mana = format!("{mana} {} ({})", mana_text(&one), restriction.name()).trim().to_string();
                    }
                }
            }
            d.set(format!("{b}.Mana"), mana);
            // The pool's mana by the source that produced it (CR 106.3), in the order it came: "{W} Barracks of the Thousand,
            // {W}{W} Plains" (restricted mana too, without its restriction).
            let from: Vec<String> = s
                .mana_from_of(p)
                .map(|e| format!("{} {}", mana_text(&e.pool), if e.source == ObjectId::NONE { "none".to_string() } else { game.name(e.source).to_string() }))
                .collect();
            if !from.is_empty() {
                d.list(&format!("{b}.ManaFrom"), from, false);
            }
            let names = |ids: &[ObjectId]| ids.iter().map(|&id| game.name(id).to_string()).collect::<Vec<_>>();
            d.list(&format!("{b}.Hand"), names(s.zone(Zone::hand(p))), false);
            // Cards this player revealed, in order, from the history (`Event::Revealed`): a beheld card (CR 701.4a), a spliced one.
            let revealed: Vec<String> = game
                .history()
                .all()
                .iter()
                .filter_map(|r| match r.event {
                    Event::Revealed { player, card } if player == p => Some(game.name(card).to_string()),
                    _ => None,
                })
                .collect();
            if !revealed.is_empty() {
                d.list(&format!("{b}.Revealed"), revealed, false);
            }
            // The crimes this player committed (CR 700.13), in order, from the history (`Event::CrimeCommitted`): each the
            // spell, or the source of the ability, that was one.
            let crimes: Vec<String> = game
                .history()
                .all()
                .iter()
                .filter_map(|r| match r.event {
                    Event::CrimeCommitted { player, by } if player == p => Some(game.name(by).to_string()),
                    _ => None,
                })
                .collect();
            if !crimes.is_empty() {
                d.list(&format!("{b}.Crimes"), crimes, true);
            }
            d.list(&format!("{b}.Library"), names(s.zone(Zone::library(p))), true);
            // The cards the player owns outside the game (CR 400.11a), in the order the sideboard lists them.
            d.list(&format!("{b}.Sideboard"), names(s.zone(Zone::sideboard(p))), true);
            let mut graveyard = names(s.zone(Zone::graveyard(p)));
            graveyard.reverse();
            d.list(&format!("{b}.Graveyard"), graveyard, true);
            // The power and toughness of a creature card in a hand, a graveyard or a library (CR 604.3: a characteristic-defining
            // ability works in every zone), `Bot1.Graveyard.Sandman, Shifting Scoundrel.Power`, and its subtypes (`.Subtypes`, absent
            // with none: what a static ability that works in every zone made it, Arcane Adaptation's chosen type); a name held twice
            // is `Name#1`, `Name#2` in zone order (a library's top first, a graveyard's bottom first). Every card there has
            // `.Colors` (empty when colorless): an effect may change a card's colors outside the battlefield (Mycosynth Lattice's
            // "all cards that aren't on the battlefield ... are colorless"), and outside the game (CR 400.11c) only its own
            // characteristic-defining abilities do.
            for (zone, label) in [(Zone::hand(p), "Hand"), (Zone::graveyard(p), "Graveyard"), (Zone::library(p), "Library"), (Zone::sideboard(p), "Sideboard")] {
                let ids = s.zone(zone).to_vec();
                for &id in &ids {
                    let c = game.chars(id);
                    let outside = zone.is(ZoneKind::Sideboard);
                    let name = game.name(id);
                    let same: Vec<ObjectId> = ids.iter().copied().filter(|&o| game.name(o) == name).collect();
                    let key = if same.len() == 1 {
                        format!("{b}.{label}.{name}")
                    } else {
                        format!("{b}.{label}.{name}#{}", same.iter().position(|&o| o == id).unwrap() + 1)
                    };
                    let mut colors: Vec<String> = c.colors.iter().map(|c| c.name().to_string()).collect();
                    colors.sort();
                    d.list(&format!("{key}.Colors"), colors, true);
                    if outside || !c.is(CardType::Creature) {
                        continue;
                    }
                    d.set(format!("{key}.Power"), c.power.unwrap_or(0));
                    d.set(format!("{key}.Toughness"), c.toughness.unwrap_or(0));
                    let subtypes = subtype_names(&c);
                    if !subtypes.is_empty() {
                        d.list(&format!("{key}.Subtypes"), subtypes, true);
                    }
                }
            }

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
                if obj.token {
                    d.set(format!("{key}.Token"), true);
                }
                // A Room (CR 709.5): its unlocked doors by name, and the mana value they leave it, whatever the doors are.
                let room = game.cards.get(c.card);
                if room.is_room() {
                    let doors: Vec<String> = room.unlocked_doors(obj.marks.doors).map(|half| half.name.to_string()).collect();
                    d.list(&format!("{key}.Doors"), doors, true);
                    d.set(format!("{key}.ManaValue"), game.mana_value(id));
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
                if obj.marks.solved {
                    d.set(format!("{key}.Solved"), true);
                }
                if game.is_suspected(id) {
                    d.set(format!("{key}.Suspected"), true);
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
                // Its computed supertypes (CR 205.4), absent when it has none: `Legendary`, `Basic; Snow`.
                let supertypes: Vec<String> = [(c.supertypes.basic, "Basic"), (c.supertypes.legendary, "Legendary"), (c.supertypes.snow, "Snow")]
                    .into_iter()
                    .filter(|&(has, _)| has)
                    .map(|(_, name)| name.to_string())
                    .collect();
                if !supertypes.is_empty() {
                    d.list(&format!("{key}.Supertypes"), supertypes, true);
                }
                let subtypes = subtype_names(&c);
                if !subtypes.is_empty() {
                    d.list(&format!("{key}.Subtypes"), subtypes, true);
                }
                let mut colors: Vec<String> = c.colors.iter().map(|c| c.name().to_string()).collect();
                if !colors.is_empty() {
                    colors.sort();
                    d.list(&format!("{key}.Colors"), colors, true);
                }
                let mut keywords: Vec<String> =
                    (0..64u8).filter(|&k| c.keywords.0 & (1u64 << k) != 0).map(|k| keyword_name(k).to_string()).collect();
                if !keywords.is_empty() {
                    keywords.sort();
                    d.list(&format!("{key}.Keywords"), keywords, true);
                }
                // Its "hexproof from [quality]" abilities (CR 702.11d), as the qualities' words: `Black; White`, `Monocolored`.
                if !c.hexproof_from.is_empty() {
                    let h = c.hexproof_from;
                    let mut qualities: Vec<String> = h.colors().iter().map(|c| c.name().to_string()).collect();
                    if h.has(HexproofFrom::MONOCOLORED) {
                        qualities.push("Monocolored".to_string());
                    }
                    if h.has(HexproofFrom::MULTICOLORED) {
                        qualities.push("Multicolored".to_string());
                    }
                    qualities.extend(h.types().iter().map(|t| t.name().to_string()));
                    qualities.sort();
                    d.list(&format!("{key}.HexproofFrom"), qualities, true);
                }
                // Its protection from colors (CR 702.16a), printed or granted, as the colors' words: `Black; Red`.
                if !c.protection.is_empty() {
                    let mut colors: Vec<String> = c.protection.iter().map(|c| c.name().to_string()).collect();
                    colors.sort();
                    d.list(&format!("{key}.ProtectionFrom"), colors, true);
                }
                if c.is(CardType::Creature) {
                    d.set(format!("{key}.Power"), c.power.unwrap_or(0));
                    d.set(format!("{key}.Toughness"), c.toughness.unwrap_or(0));
                    d.set(format!("{key}.Damage"), obj.damage);
                    d.set(format!("{key}.SummoningSick"), summoning_sick(game.cards, s, &game.log, id));
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
        for e in expectations.iter().filter(|e| !e.is_client()) {
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
