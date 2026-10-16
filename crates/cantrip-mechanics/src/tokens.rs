//! The tokens the rules define by name (CR 111.10), as the C# `PredefinedTokens` names them.

use cantrip_api::*;
use crate::statics::itself;

fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// A colorless Clue artifact token with "{2}, Sacrifice this token: Draw a card." (CR 111.10f).
pub static CLUE: CardDef = CardDef {
    name: "Clue Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Clue]),
    abilities: &[ability(cost("{2}").and(SACRIFICE_THIS), draw_a_card, "{2}, Sacrifice this token: Draw a card.")],
    text: "{2}, Sacrifice this token: Draw a card.",
    ..CardDef::BLANK
};

fn add_one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

/// A colorless Treasure artifact token with "{T}, Sacrifice this artifact: Add one mana of any color." (CR 111.10a).
pub static TREASURE: CardDef = CardDef {
    name: "Treasure Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Treasure]),
    abilities: &[mana_ability(TAP.and(SACRIFICE_THIS), add_any_color(1).by(add_one_of_any_color), "{T}, Sacrifice this artifact: Add one mana of any color.")],
    text: "{T}, Sacrifice this artifact: Add one mana of any color.",
    ..CardDef::BLANK
};

/// Creates a Treasure token (CR 111.10a) under `controller`.
pub fn create_treasure(g: &mut dyn Game, controller: PlayerId) -> Result<(), Suspend> {
    let treasure = g.card_id(&TREASURE);
    g.create_tokens(treasure, controller, 1, false)?;
    Ok(())
}

fn any_card(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

/// A colorless Blood artifact token with "{1}, {T}, Discard a card, Sacrifice this token: Draw a card." (CR 111.10g).
pub static BLOOD: CardDef = CardDef {
    name: "Blood Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Blood]),
    abilities: &[ability(cost("{1}, {T}").and(SACRIFICE_THIS).and(discard(1, any_card)), draw_a_card, "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.")],
    text: "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.",
    ..CardDef::BLANK
};

/// Investigate (CR 701.16a): create a Clue token. Used by Thraben Inspector and Tireless Tracker.
pub fn investigate(g: &mut dyn Game, controller: PlayerId) -> Result<(), Suspend> {
    let clue = g.card_id(&CLUE);
    g.create_tokens(clue, controller, 1, false)?;
    Ok(())
}

fn put_plus_one_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(Target::Object(creature)) = item.targets().first().copied() {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// A colorless Mutagen artifact token with "{1}, {T}, Sacrifice this token: Put a +1/+1 counter on target creature.
/// Activate only as a sorcery." (CR 111.10v). Michelangelo, Weirdness to 11 and Mutagen Man print it.
pub static MUTAGEN: CardDef = CardDef {
    name: "Mutagen Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Mutagen]),
    abilities: &[ability(cost("{1}, {T}").and(SACRIFICE_THIS), put_plus_one_counter, "{1}, {T}, Sacrifice this token: Put a +1/+1 counter on target creature. Activate only as a sorcery.")
        .sorcery_speed()
        .targets(&[crate::targets::CREATURE])],
    text: "{1}, {T}, Sacrifice this token: Put a +1/+1 counter on target creature. Activate only as a sorcery.",
    ..CardDef::BLANK
};

fn explore_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Object(creature) = item.targets[0]
    {
        crate::explore(g, creature)?;
    }
    Ok(())
}

/// A colorless 2/2 Robot artifact creature token. Melded Moxite and Pinnacle Starcage create it. It shares its name with
/// the 1/1 Robot (Ravenous Robots'), so the card table keeps the definitions apart.
pub static ROBOT_2_2: CardDef = CardDef {
    name: "Robot",
    has_cost: false,
    colors: Some(ColorSet::NONE),
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Robot]),
    power: Some(2),
    toughness: Some(2),
    shares_name: true,
    ..CardDef::BLANK
};

/// A colorless Map artifact token with "{1}, {T}, Sacrifice this token: Target creature you control explores.
/// Activate only as a sorcery." (CR 111.10s). Fanatical Offering and Seasoned Dungeoneer make it.
pub static MAP: CardDef = CardDef {
    name: "Map Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Map]),
    abilities: &[ability(cost("{1}, {T}").and(SACRIFICE_THIS), explore_target, "{1}, {T}, Sacrifice this token: Target creature you control explores. Activate only as a sorcery.")
        .sorcery_speed()
        .targets(&[crate::targets::CREATURE_YOU_CONTROL])],
    text: "{1}, {T}, Sacrifice this token: Target creature you control explores. Activate only as a sorcery.",
    ..CardDef::BLANK
};

/// Creates a Map token (CR 111.10s) under `controller`.
pub fn create_map(g: &mut dyn Game, controller: PlayerId) -> Result<(), Suspend> {
    let map = g.card_id(&MAP);
    g.create_tokens(map, controller, 1, false)?;
    Ok(())
}

fn gain_three_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    Ok(())
}

/// A colorless Food artifact token with "{2}, {T}, Sacrifice this token: You gain 3 life." (CR 111.10b). Generous Ent
/// and Oko, Thief of Crowns create it.
pub static FOOD: CardDef = CardDef {
    name: "Food Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Food]),
    abilities: &[ability(cost("{2}, {T}").and(SACRIFICE_THIS), gain_three_life, "{2}, {T}, Sacrifice this token: You gain 3 life.")],
    text: "{2}, {T}, Sacrifice this token: You gain 3 life.",
    ..CardDef::BLANK
};

/// Creates a Food token (CR 111.10b) under `controller`.
pub fn create_food(g: &mut dyn Game, controller: PlayerId) -> Result<(), Suspend> {
    let food = g.card_id(&FOOD);
    g.create_tokens(food, controller, 1, false)?;
    Ok(())
}

/// A 1/1 white Monk creature token with prowess. Cori-Steel Cutter's flurry and Monastery Mentor make it.
pub static MONK: CardDef = CardDef {
    name: "Monk",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Monk]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[crate::triggers::PROWESS],
    text: "Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)",
    ..CardDef::BLANK
};

/// The 0/0 black Phyrexian Germ creature token living weapon makes (CR 702.92a): Nettlecyst, Kaldra Compleat.
pub static GERM: CardDef = CardDef {
    name: "Phyrexian Germ",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Black)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Phyrexian, Subtype::Germ]),
    power: Some(0),
    toughness: Some(0),
    ..CardDef::BLANK
};

/// A colorless 3/3 Golem artifact creature token. Legion Extruder and Sandstorm Salvager make it.
pub static GOLEM: CardDef = CardDef {
    name: "Golem",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Golem]),
    power: Some(3),
    toughness: Some(3),
    ..CardDef::BLANK
};

/// The 1/1 colorless Pilot creature token: "This token crews Vehicles as though its power were 2 greater." Shorikai,
/// Genesis Engine and Reckoner Bankbuster make it.
pub static PILOT: CardDef = CardDef {
    name: "Pilot",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Pilot]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::CrewsAsThoughPowerTwoGreater]),
    text: "This token crews Vehicles as though its power were 2 greater.",
    ..CardDef::BLANK
};

/// "This token gets +1/+1 for each artifact you control" (layer 7c): a modification of its printed 0/0, so a lone
/// Construct counts itself and is a 1/1. "You" is its controller now.
fn construct_size(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let you = state.obj(source).controller;
    let n = state
        .battlefield()
        .iter()
        .filter(|&&id| state.obj(id).controller == you && view.chars(id).is(CardType::Artifact))
        .count() as i16;
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

/// The 0/0 colorless Construct artifact creature token with "This token gets +1/+1 for each artifact you control". Urza's Saga
/// and Urza, Lord High Artificer make it.
pub static CONSTRUCT: CardDef = CardDef {
    name: "Construct",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Construct]),
    power: Some(0),
    toughness: Some(0),
    statics: &[static_ability(Layer::PtModify, itself, construct_size, "This token gets +1/+1 for each artifact you control.")],
    text: "This token gets +1/+1 for each artifact you control.",
    ..CardDef::BLANK
};

/// The 0/0 colorless Phyrexian artifact creature an Incubator transforms into (CR 111.10i).
pub static PHYREXIAN: CardDef = CardDef {
    name: "Phyrexian Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Phyrexian]),
    power: Some(0),
    toughness: Some(0),
    ..CardDef::BLANK
};

fn transform_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.transform(item.object);
    Ok(())
}

/// The double-faced Incubator artifact token (CR 111.10i, 701.53b): "{2}: Transform this token." Its counters are
/// those it was created with (`incubate`). A user lists this and `PHYREXIAN` in its `EXTRA`.
pub static INCUBATOR: CardDef = CardDef {
    name: "Incubator Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Incubator]),
    abilities: &[ability(cost("{2}"), transform_it, "{2}: Transform this token.")],
    back: Some(&PHYREXIAN),
    text: "{2}: Transform this token.",
    ..CardDef::BLANK
};

/// Incubate N (CR 701.53a): create an Incubator token with N +1/+1 counters on it. Sunfall and Metamorphosis-style
/// incubators share it.
pub fn incubate(g: &mut dyn Game, controller: PlayerId, n: i16) -> Result<(), Suspend> {
    let incubator = g.card_id(&INCUBATOR);
    g.create_tokens_with_counters(incubator, controller, 1, false, &[(CounterKind::PlusOne, n)])?;
    Ok(())
}

/// "When this token dies, you gain 1 life."
fn gain_a_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

/// The 1/1 black and green Pest creature token with "When this token dies, you gain 1 life."
pub static PEST: CardDef = CardDef {
    name: "Pest",
    has_cost: false,
    colors: Some(ColorSet(0b10100)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Pest]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[triggered(crate::triggers::THIS_DIES, gain_a_life, "When this token dies, you gain 1 life.")
        .look_back()],
    text: "When this token dies, you gain 1 life.",
    ..CardDef::BLANK
};

/// A colorless Gold artifact token with "Sacrifice this artifact: Add one mana of any color." (CR 111.10c).
pub static GOLD: CardDef = CardDef {
    name: "Gold Token",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Gold]),
    abilities: &[mana_ability(SACRIFICE_THIS, add_any_color(1).by(add_one_of_any_color), "Sacrifice this artifact: Add one mana of any color.")],
    text: "Sacrifice this artifact: Add one mana of any color.",
    ..CardDef::BLANK
};

/// A red and green Heartwood artifact token with "{T}: Add {R} or {G}." (CR 111.10x). Aerid Konstrari makes it.
pub static HEARTWOOD: CardDef = CardDef {
    name: "Heartwood Token",
    has_cost: false,
    colors: Some(ColorSet(0b11000)),
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Heartwood]),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Red, Color::Green]), "{T}: Add {R} or {G}.")],
    text: "{T}: Add {R} or {G}.",
    ..CardDef::BLANK
};

/// Creates a Heartwood token under `controller`.
pub fn create_heartwood(g: &mut dyn Game, controller: PlayerId) -> Result<(), Suspend> {
    create(g, controller, &HEARTWOOD, 1)?;
    Ok(())
}

/// Creates `count` tokens of `def` under `controller` (CR 111.1), untapped: the card file lists `def` in its `EXTRA`. The
/// tokens are created together (CR 101.4). Every card that makes a Soldier, a Servo or a Rebel calls it, as do those that
/// make a token of their own.
pub fn create(g: &mut dyn Game, controller: PlayerId, def: &'static CardDef, count: u32) -> Result<Vec<ObjectId>, Suspend> {
    let def = g.card_id(def);
    g.create_tokens(def, controller, count, false)
}

/// Creates a Blood token (CR 111.10g) under `controller`. Bloodtithe Harvester and Ivora, Insatiable Heir make it.
pub fn create_blood(g: &mut dyn Game, controller: PlayerId) -> Result<(), Suspend> {
    create(g, controller, &BLOOD, 1)?;
    Ok(())
}

/// Gives `player` the emblem `def` (CR 114.2): an object in the command zone that nothing moves. The planeswalker lists
/// `def` in its `EXTRA`. Wrenn and Six and Teferi, Hero of Dominaria make theirs so.
pub fn create_emblem(g: &mut dyn Game, player: PlayerId, def: &'static CardDef) -> ObjectId {
    let def = g.card_id(def);
    g.create_emblem(def, player)
}

/// The 1/1 white Soldier creature token. Hero of Bladehold, Elspeth, Storm Slayer and Torsten, Founder of Benalia make
/// it. (Third Path Iconoclast's is an artifact Soldier of the same name.)
pub static SOLDIER: CardDef = CardDef {
    name: "Soldier",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Soldier]),
    power: Some(1),
    toughness: Some(1),
    shares_name: true,
    ..CardDef::BLANK
};

/// The 2/2 red Rebel creature token. Glimmer Lens and Otharri, Suns' Glory make it.
pub static REBEL: CardDef = CardDef {
    name: "Rebel",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Rebel]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

/// The 1/1 colorless Servo artifact creature token. Retrofitter Foundry and Saheeli, Sublime Artificer make it.
pub static SERVO: CardDef = CardDef {
    name: "Servo",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Servo]),
    power: Some(1),
    toughness: Some(1),
    shares_name: true,
    ..CardDef::BLANK
};

/// A 1/1 blue Fish creature token (CR 702.174f). Into the Flood Maw's gift makes it, tapped.
pub static FISH: CardDef = CardDef {
    name: "Fish",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Blue)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Fish]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// An 8/8 blue Octopus creature token (CR 702.174i). "Gift an Octopus" makes it.
pub static OCTOPUS: CardDef = CardDef {
    name: "Octopus",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Blue)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Octopus]),
    power: Some(8),
    toughness: Some(8),
    ..CardDef::BLANK
};
