use cantrip_api::*;
use cantrip_mechanics::{effects, tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, create a 1/1 colorless Hero creature token. If four or more mana was spent to cast that
/// spell, draw two cards. If eight or more mana was spent to cast that spell, sacrifice Tellah and it deals that much damage to each
/// opponent" (CR 601.2h, 608.2c). The mana spent is read from the spell the ability triggered on, so it resolves the same if that spell
/// was countered or left the stack. Each step is done whether or not the others could be: Tellah that has left the battlefield is not
/// sacrificed again, and still deals the damage as it last existed (CR 113.7a).
fn hero_draw_and_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let spent = g.obj(spell).spent.total();
    tokens::create(g, item.controller, &tokens::HERO_1_1_COLORLESS, 1)?;
    if spent >= 4 {
        g.draw(item.controller, 2);
    }
    if spent >= 8 {
        g.sacrifice(item.controller, &[item.object]);
        effects::damage_each_opponent(g, item, spent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::NONCREATURE_SPELL_YOU_CAST_ABOUT_IT,
        hero_draw_and_sacrifice,
        "Whenever you cast a noncreature spell, create a 1/1 colorless Hero creature token. If four or more mana was spent to cast that spell, draw two cards. If eight or more mana was spent to cast that spell, sacrifice Tellah and it deals that much damage to each opponent.",
    )],
    ..oracle::TELLAH_GREAT_SAGE
};

pub static EXTRA: &[&CardDef] = &[&tokens::HERO_1_1_COLORLESS];
