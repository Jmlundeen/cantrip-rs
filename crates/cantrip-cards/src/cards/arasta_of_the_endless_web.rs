use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Create a 1/2 green Spider creature token with reach" (CR 111.1).
fn create_spider(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::SPIDER_1_2, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::SPELL_CAST, triggers::opponent_casts_instant_or_sorcery), create_spider, "Whenever an opponent casts an instant or sorcery spell, create a 1/2 green Spider creature token with reach.")],
    ..oracle::ARASTA_OF_THE_ENDLESS_WEB
};

pub static EXTRA: &[&CardDef] = &[&tokens::SPIDER_1_2];
