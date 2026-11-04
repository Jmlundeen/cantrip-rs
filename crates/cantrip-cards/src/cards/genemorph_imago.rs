use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{lands_you_control, targets};
use cantrip_oracle as oracle;

/// "Target creature has base power and toughness 3/3 until end of turn. If you control six or more lands, that creature has base power
/// and toughness 6/6 until end of turn instead" (CR 613.4b, 608.2c): the lands are counted as the ability resolves, the one that
/// entered among them; only one of the two effects is made, the target having to be legal still (CR 608.2b).
fn base_three_three_or_six_six(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (power, toughness) = if lands_you_control(g, item.controller) >= 6 { (6, 6) } else { (3, 3) };
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::SetPt(power, toughness), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        base_three_three_or_six_six,
        "Landfall \u{2014} Whenever a land you control enters, target creature has base power and toughness 3/3 until end of turn. If you control six or more lands, that creature has base power and toughness 6/6 until end of turn instead.",
    )
    .targets(&[targets::CREATURE])],
    ..oracle::GENEMORPH_IMAGO
};
