use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::tokens::{self, TREASURE};
use cantrip_mechanics::{choose, targets, triggers};
use cantrip_oracle as oracle;

/// When this creature enters, create a Treasure token.
fn treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &TREASURE, 1)?;
    Ok(())
}

/// You may sacrifice one or more Treasures (any Treasure you control, token or not), together (CR 701.21). When you do
/// (CR 603.12), the reflexive ability triggers counting them: "that many" is the number sacrificed, and none sacrificed
/// is not doing it.
fn sacrifice_treasures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let treasures: Vec<ObjectId> = cantrip_mechanics::tokens::treasures_controlled(g, you);
    let chosen = choose::any_number_of(g, you, item.object, Purpose::Sacrifice, "Treasure", treasures.clone(), treasures)?;
    let sacrificed = g.sacrifice(&chosen);
    if !sacrificed.is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger_counted(item.object, you, AbilityRef::delayed(this, 0), Target::None, u16::try_from(sacrificed.len()).unwrap_or(u16::MAX));
    }
    Ok(())
}

/// Up to that many target creatures gain double strike until end of turn: each target still legal (CR 608.2b).
fn double_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = targets::legal_objects(g, item);
    instruction(g, |g| {
        for creature in creatures {
            effects::gain_until_end_of_turn(g, item.object, creature, Keyword::DoubleStrike);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, treasure, "When this creature enters, create a Treasure token."),
        triggered(triggers::YOU_ATTACK, sacrifice_treasures, "Whenever you attack, you may sacrifice one or more Treasures. When you do, up to that many target creatures gain double strike until end of turn."),
    ],
    delayed: &[delayed_trigger(double_strike, "When you do, up to that many target creatures gain double strike until end of turn.")
        .targets(&[targets::CREATURE.between(0, TargetSpec::COUNTED)])],
    ..oracle::SWASHBUCKLER_EXTRAORDINAIRE
};

pub static EXTRA: &[&CardDef] = &[&TREASURE];
