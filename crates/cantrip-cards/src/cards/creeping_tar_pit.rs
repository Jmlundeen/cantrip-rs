use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

/// "Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be
/// blocked this turn."
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let colors = ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0);
    lands::becomes_creature_until_end_of_turn(g, item, (3, 2), colors, Subtype::Elemental, Keywords::NONE);
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::CantBeBlocked])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Blue, Color::Black]), "{T}: Add {U} or {B}."),
        ability(cost("{1}{U}{B}"), becomes_creature, "{1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be blocked this turn."),
    ],
    ..oracle::CREEPING_TAR_PIT
};
