use cantrip_api::*;
use cantrip_mechanics::tokens::{self, SPIDER};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::web_slinging;
use cantrip_oracle as oracle;

/// "If they were cast using web-slinging" (CR 603.4, 702.188a): an intervening "if", checked as the ability triggers and
/// again as it resolves, about how the spell this permanent was came to be cast.
fn cast_using_web_slinging(q: &dyn Query, asker: &Asker) -> bool {
    web_slinging::was_cast_using_web_slinging(q, asker.source)
}

/// "You gain 3 life and create two 2/1 green Spider creature tokens with reach."
fn gain_and_spiders(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    tokens::create(g, item.controller, &SPIDER, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{4}{G}{G}"), "Web-slinging {4}{G}{G}")],
    abilities: &[triggered(
        THIS_ENTERS,
        gain_and_spiders,
        "When Spiders-Man enters, if they were cast using web-slinging, you gain 3 life and create two 2/1 green Spider creature tokens with reach.",
    )
    .condition(cast_using_web_slinging)],
    ..oracle::SPIDERS_MAN_HEROIC_HORDE
};

pub static EXTRA: &[&CardDef] = &[&SPIDER];
