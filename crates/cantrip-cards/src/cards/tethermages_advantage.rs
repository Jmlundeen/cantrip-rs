use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+2 and gains reach until end of turn. Untap it."
fn buff_and_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        instruction(g, |g| {
            pump(g, item.object, creature, 2, 2);
            effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Reach);
        });
        g.end_sentence();
        g.untap(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], buff_and_untap), ..oracle::TETHERMAGES_ADVANTAGE };
