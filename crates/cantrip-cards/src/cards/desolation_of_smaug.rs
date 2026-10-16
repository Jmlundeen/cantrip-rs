use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Desolation of Smaug deals 3 damage to each non-Dragon creature." Every creature on the battlefield that isn't a Dragon
/// as the effect is carried out (CR 608.2c), dealt at once (CR 120.2). "Add four mana in any combination of colors. Spend
/// this mana only to cast Dragon spells." The four colors are chosen as the spell resolves, repeats allowed (CR 106.1b),
/// and the mana pays only for casting a spell with the subtype Dragon (CR 106.6, 205.3; its ruling: Dragon's Hoard is
/// no Dragon spell). "Dragon" is a word of the spell's text, read through its words both times (CR 612).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let dragon = g.words(item.object).subtype(Subtype::Dragon);
    let hits: Vec<(ObjectId, Target, u16)> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) && !chars.has_subtype(dragon)
        })
        .map(|id| (item.object, Target::Object(id), 3))
        .collect();
    g.damage_together(&hits);
    for color in cantrip_mechanics::choose_colors(g, item.controller, item.object, ColorSet(0b11111), 4)? {
        g.add_restricted_mana(item.controller, color.mana(), 1, ManaRestriction::Subtypes(SubtypeSpells::of(dragon)));
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::DESOLATION_OF_SMAUG };
