use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This creature gains deathtouch until end of turn" (CR 702.2, 611.2a): a layer 6 keyword for the rest of the turn.
fn gain_deathtouch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Deathtouch])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(cost("{2}"), gain_deathtouch, "{2}: This creature gains deathtouch until end of turn."),
    ],
    ..oracle::POISON_DART_FROG
};
