use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::{choose, choose_from_hand, damage_target, haste_until_end_of_turn, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Target Vehicle you control" (CR 115.1, 301.7): a permanent with the Vehicle subtype that its controller controls.
fn vehicle_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) == asker.controller && q.chars(id).has_subtype(Subtype::Vehicle))
}

/// "At the beginning of combat on your turn, choose up to one target Vehicle you control. Until end of turn, it becomes an
/// artifact creature and gains haste." Two continuous effects from this ability's source (CR 611.2a, 613.1d): the types it
/// gains, and haste.
fn animate_vehicle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(vehicle) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, vehicle, EffectKind::AddTypes(TypeSet::of(&[CardType::Artifact, CardType::Creature])), Duration::EndOfTurn);
    haste_until_end_of_turn(g, item.object, vehicle);
    Ok(())
}

/// The options of the +2, as `labels` below.
const SACRIFICE: u8 = 0;
const DISCARD: u8 = 1;
const NEITHER: u8 = 2;

/// +2: "You may sacrifice an artifact or discard a card. If you do, draw a card." (CR 608.2c-d). The player chooses as the
/// ability resolves which of the two to do, or neither; an option with nothing to do it with (no artifact, an empty hand) is
/// not offered. The card is drawn only if the sacrifice or the discard actually happened (CR 608.2c).
fn sacrifice_or_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let artifacts: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Artifact)).collect();
    let mut options = Vec::new();
    if !artifacts.is_empty() {
        options.push(SACRIFICE);
    }
    if !g.state().zone(Zone::hand(you)).is_empty() {
        options.push(DISCARD);
    }
    if options.is_empty() {
        return Ok(());
    }
    options.push(NEITHER);
    let card = g.obj(item.object).card;
    let labels = options.iter().map(|&i| Label::Card(card, i)).collect();
    let done = match options[choose::one_of(g, you, item.object, labels)? as usize] {
        SACRIFICE => {
            let chosen = choose::cards(g, you, item.object, Purpose::Sacrifice, artifacts, 1, false)?;
            !g.sacrifice(&chosen).is_empty()
        }
        DISCARD => {
            let chosen = choose_from_hand(g, you, item.object, 1)?;
            !g.discard_together(&chosen).is_empty()
        }
        _ => false,
    };
    if done {
        g.draw(you, 1);
    }
    Ok(())
}

/// 0: "Create a 3/2 colorless Vehicle artifact token with crew 1."
fn create_vehicle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let vehicle = g.card_id(&VEHICLE);
    g.create_tokens(vehicle, item.controller, 1, false)?;
    Ok(())
}

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

/// The 3/2 colorless Vehicle artifact token with crew 1.
pub static VEHICLE: CardDef = CardDef {
    name: "Vehicle",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Vehicle]),
    power: Some(3),
    toughness: Some(2),
    abilities: &[crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)")],
    text: "Crew 1",
    ..CardDef::BLANK
};

fn three_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

/// The emblem: "Whenever an artifact you control enters, this emblem deals 3 damage to any target." The emblem is the damage's
/// source (CR 114.4).
pub static EMBLEM: CardDef = CardDef {
    name: "Chandra, Spark Hunter emblem",
    has_cost: false,
    abilities: &[triggered(
        triggers::ARTIFACT_YOU_CONTROL_ENTERS,
        three_damage,
        "Whenever an artifact you control enters, this emblem deals 3 damage to any target.",
    )
    .targets(&[targets::ANY])],
    text: "Whenever an artifact you control enters, this emblem deals 3 damage to any target.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::BEGINNING_OF_YOUR_COMBAT,
            animate_vehicle,
            "At the beginning of combat on your turn, choose up to one target Vehicle you control. Until end of turn, it becomes an artifact creature and gains haste.",
        )
        .targets(&[up_to_one(vehicle_you_control)]),
        ability(loyalty(2), sacrifice_or_discard, "+2: You may sacrifice an artifact or discard a card. If you do, draw a card."),
        ability(loyalty(0), create_vehicle, "0: Create a 3/2 colorless Vehicle artifact token with crew 1."),
        ability(loyalty(-7), emblem, "\u{2212}7: You get an emblem with \"Whenever an artifact you control enters, this emblem deals 3 damage to any target.\""),
    ],
    labels: &["Sacrifice an artifact", "Discard a card", "Neither"],
    ..oracle::CHANDRA_SPARK_HUNTER
};

pub static EXTRA: &[&CardDef] = &[&VEHICLE, &EMBLEM];
