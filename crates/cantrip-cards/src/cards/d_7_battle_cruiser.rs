use cantrip_api::*;
use cantrip_mechanics::{delayed, station, statics, tokens, triggers};
use cantrip_mechanics::tokens::KLINGON;
use cantrip_oracle as oracle;

/// The 8+ striation makes this a 3/3 artifact creature with flying while it has eight charge counters (CR 721.2a-b).
const EIGHT_PLUS: &str = "8+ | Flying";

/// "At the beginning of combat on your turn, create a 2/1 red Klingon Warrior creature token with haste. Sacrifice it at
/// the beginning of the next end step." The token is carried to its delayed trigger by object identity (CR 603.7c, 400.7).
fn create_klingon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let created = tokens::create(g, item.controller, &KLINGON, 1)?;
    delayed::at_next_end_step_about(g, item, 0, &created);
    Ok(())
}

/// The 3+ striation's condition is checked both as the trigger would trigger and as it resolves (CR 603.4, 721.2a).
fn charged_three(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).counters(CounterKind::Charge) >= 3
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
        triggered(triggers::BEGINNING_OF_YOUR_COMBAT, create_klingon, "At the beginning of combat on your turn, create a 2/1 red Klingon Warrior creature token with haste. Sacrifice it at the beginning of the next end step.")
            .condition(charged_three),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, station::charged::<8>, statics::have_flying, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<8>, statics::base_pt::<3, 3>, EIGHT_PLUS).continues(),
    ],
    delayed: &[delayed::SACRIFICE_CREATED],
    ..oracle::D_7_BATTLE_CRUISER
};

pub static EXTRA: &[&CardDef] = &[&KLINGON];
