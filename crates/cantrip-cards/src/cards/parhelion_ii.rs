use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::tokens::ANGEL;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever Parhelion II attacks, create two 4/4 white Angel creature tokens with flying and vigilance that are
/// attacking": their controller chooses what each attacks as it enters (CR 508.4), and they were never declared as
/// attackers, so nothing that triggers on attacking creatures sees them.
fn angels(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let angel = g.card_id(&ANGEL);
    create_attacking_of_your_choice(g, item.controller, item.object, angel, 2, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, angels, "Whenever Parhelion II attacks, create two 4/4 white Angel creature tokens with flying and vigilance that are attacking."),
        crew(4, "Crew 4 (Tap any number of creatures you control with total power 4 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::PARHELION_II
};

pub static EXTRA: &[&CardDef] = &[&ANGEL];
