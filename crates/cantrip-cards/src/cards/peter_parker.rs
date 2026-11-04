use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, tokens, web_slinging};
use cantrip_oracle as oracle;

/// "Create a 2/1 green Spider creature token with reach", under the ability's controller.
fn create_spider(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::SPIDER, 1)?;
    Ok(())
}

/// "Each legendary spell you cast that's one or more colors has web-slinging {G}{W}{U}" (CR 702.188a, 113.6, 604): a legendary
/// card with at least one color (a colorless legendary spell has none), judged as the spell it would be in the zone it is cast
/// from, whichever that is. Only the spells this permanent's controller casts.
fn colored_legendary_spell_you_cast(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> bool {
    let chars = q.chars(card);
    caster == asker.controller && chars.supertypes.legendary && !chars.colors.is_empty()
}

pub static BACK: CardDef = CardDef {
    spell_alternatives: &[web_slinging::granted(colored_legendary_spell_you_cast, mana("{G}{W}{U}"), "Web-slinging {G}{W}{U}")],
    ..oracle::AMAZING_SPIDER_MAN
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_spider, "When Peter Parker enters, create a 2/1 green Spider creature token with reach."),
        ability(cost("{1}{G}{W}{U}"), effects::transform_self, "{1}{G}{W}{U}: Transform Peter Parker. Activate only as a sorcery.").sorcery_speed(),
    ],
    back: Some(&BACK),
    ..oracle::PETER_PARKER
};

pub static EXTRA: &[&CardDef] = &[&tokens::SPIDER];
