use cantrip_api::*;
use cantrip_mechanics::exile_all_until_leaves;
use cantrip_mechanics::tokens::{self, ROBOT_2_2};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Exile all artifacts and creatures with mana value 2 or less until this artifact leaves the battlefield" (CR 610.3):
/// every one on the battlefield, whoever controls it, exiled together; the ones on the battlefield are chosen as the
/// ability resolves, by mana value as the game reads it (a token's is 0, an {X} permanent's X is 0).
fn exile_cheap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_all_until_leaves(g, item, |g, id| {
        let c = g.chars(id);
        (c.is(CardType::Artifact) || c.is(CardType::Creature)) && g.mana_value(id) <= 2
    });
    Ok(())
}

/// "Put each card exiled with this artifact into its owner's graveyard, then create a 2/2 colorless Robot artifact
/// creature token for each card put into a graveyard this way. Sacrifice this artifact." (CR 607.2a): a card a
/// replacement effect sent elsewhere is not put into a graveyard, so it makes no Robot.
fn release(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = g.exiled_with(g.linked(item));
    let moved = g.put_into_graveyard(&exiled);
    let count = moved.iter().filter(|&&id| g.obj(id).zone.is(ZoneKind::Graveyard)).count();
    if count > 0 {
        tokens::create(g, item.controller, &ROBOT_2_2, count as u32)?;
    }
    g.sacrifice(item.controller, &[item.object]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_cheap, "When this artifact enters, exile all artifacts and creatures with mana value 2 or less until this artifact leaves the battlefield."),
        ability(cost("{6}{W}{W}"), release, "{6}{W}{W}: Put each card exiled with this artifact into its owner's graveyard, then create a 2/2 colorless Robot artifact creature token for each card put into a graveyard this way. Sacrifice this artifact."),
    ],
    ..oracle::PINNACLE_STARCAGE
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
