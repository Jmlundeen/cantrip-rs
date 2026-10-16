use cantrip_api::*;
use cantrip_mechanics::tokens::{self, HUMAN_CITIZEN};
use cantrip_mechanics::triggers::CREATURE_SPELL_YOU_CAST;
use cantrip_mechanics::{pump, web_slinging};
use cantrip_oracle as oracle;

/// "Create a 1/1 green and white Human Citizen creature token."
fn citizen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &HUMAN_CITIZEN, 1)?;
    Ok(())
}

/// "Creatures you control get +2/+2 and gain vigilance until end of turn": the creatures there are as it resolves
/// (CR 611.2c).
fn pump_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        pump(g, item.object, id, 2, 2);
        g.add_effect(item.object, id, EffectKind::AddKeywords(Keywords::of(&[Keyword::Vigilance])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{1}{G}{W}"), "Web-slinging {1}{G}{W}")],
    abilities: &[
        triggered(CREATURE_SPELL_YOU_CAST, citizen, "Whenever you cast a creature spell, create a 1/1 green and white Human Citizen creature token."),
        ability(cost("{3}{G}{W}"), pump_creatures, "{3}{G}{W}: Creatures you control get +2/+2 and gain vigilance until end of turn."),
    ],
    ..oracle::SILK_WEB_WEAVER
};

pub static EXTRA: &[&CardDef] = &[&HUMAN_CITIZEN];
