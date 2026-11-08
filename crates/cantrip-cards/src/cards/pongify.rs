use cantrip_api::*;
use cantrip_mechanics::{destroy_target_no_regen, targets, tokens};
use cantrip_oracle as oracle;

/// The 3/3 green Ape creature token it makes: no other printed card makes it exactly, so it lives here rather than in
/// the shared tokens table.
pub static APE: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    ..token!(Ape)
};

/// "Destroy target creature. It can't be regenerated. Its controller creates a 3/3 green Ape creature token."
/// (CR 701.8, 701.19c, 608.2b): the no-regen destroy leaves the target's controller to read even when indestructible
/// kept it on the battlefield, while an illegal target ends the spell with no token.
fn pongify(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = destroy_target_no_regen(g, item, 0) else { return Ok(()) };
    g.end_sentence();
    let controller = g.controller(creature);
    tokens::create(g, controller, &APE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], pongify),
    ..oracle::PONGIFY
};

pub static EXTRA: &[&CardDef] = &[&APE];
