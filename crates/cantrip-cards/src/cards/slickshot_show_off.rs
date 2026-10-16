use cantrip_api::*;
use cantrip_mechanics::plot::plot;
use cantrip_mechanics::pump;
use cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// "this creature gets +2/+0 until end of turn" (CR 611.2). If it has left the battlefield by the time the trigger
/// resolves, the effect does nothing.
fn pump_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 2, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(NONCREATURE_SPELL_YOU_CAST, pump_self, "Whenever you cast a noncreature spell, this creature gets +2/+0 until end of turn."),
        plot(
            mana("{1}{R}"),
            "Plot {1}{R} (You may pay {1}{R} and exile this card from your hand. Cast it as a sorcery on a later turn without paying its mana cost. Plot only as a sorcery.)",
        ),
    ],
    ..oracle::SLICKSHOT_SHOW_OFF
};
