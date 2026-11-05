use cantrip_api::*;
use cantrip_mechanics::{tokens, choose_color};
use cantrip_oracle as oracle;

/// "Create two 1/1 colorless Borg artifact creature tokens, then artifact creatures you control gain protection from the
/// color of your choice until end of turn" (CR 701.7, 611.2a, 702.16): the two tokens enter as one event, then the
/// controller chooses the color, and every artifact creature they control as that happens - the two tokens included -
/// gains protection from it. Borg Incursion is the spell.
fn incursion(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BORG, 2)?;
    g.end_sentence();
    let color = choose_color(g, item.controller, item.object)?;
    let artifact_creatures: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Artifact) && chars.is(CardType::Creature)
        })
        .collect();
    instruction(g, |g| {
        for id in artifact_creatures {
            g.add_effect(item.object, id, EffectKind::Protection(ColorSet::of(color)), Duration::EndOfTurn);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], incursion), ..oracle::BORG_INCURSION };

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
