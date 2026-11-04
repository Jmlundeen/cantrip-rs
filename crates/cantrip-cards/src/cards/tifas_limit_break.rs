use cantrip_api::*;
use cantrip_mechanics::{double, effects, targets, tiered, triple};
use cantrip_oracle as oracle;

/// Meteor Strikes: "Double target creature's power and toughness until end of turn." (CR 701.10)
fn meteor_strikes(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    double::power_and_toughness(g, item.object, creature);
    Ok(())
}

/// Final Heaven: "Triple target creature's power and toughness until end of turn." (CR 701.11)
fn final_heaven(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    triple::power_and_toughness(g, item.object, creature);
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(effects::target_creature_gets::<2, 2>, "Somersault \u{2014} {0} \u{2014} Target creature gets +2/+2 until end of turn.")
        .targets(&[targets::CREATURE])
        .cost(mana("{0}")),
    mode(meteor_strikes, "Meteor Strikes \u{2014} {2} \u{2014} Double target creature's power and toughness until end of turn.")
        .targets(&[targets::CREATURE])
        .cost(mana("{2}")),
    mode(final_heaven, "Final Heaven \u{2014} {6}{G} \u{2014} Triple target creature's power and toughness until end of turn.")
        .targets(&[targets::CREATURE])
        .cost(mana("{6}{G}")),
];

static MODAL: ModalDef = tiered::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::TIFAS_LIMIT_BREAK };
