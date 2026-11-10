//! Path of Ancestry's ruling: "If Path of Ancestry's last ability produces two mana (most likely due to Mana Reflection),
//! spending those two mana to cast creature spells that share a creature type with your commander will cause two abilities
//! to trigger. Each of those abilities will cause you to scry 1." One activation makes one "when that mana is spent"
//! ability riding both mana (CR 603.7a, 106.3), which triggers once for each of them spent (`DelayedWhen::ThatMana`). No
//! doubler is implemented, so a test-only one stands in for Mana Reflection ("If you tap a permanent for mana, it produces
//! twice as much of that mana instead", CR 106.12b, 614.1a).

use std::sync::OnceLock;

use cantrip_engine::*;
use cantrip_script::{GameScript, run_on};

/// "If you tap a permanent for mana": a production the replacement's controller taps for.
fn you_tap_for_mana(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ManaProduction(m) if m.player == asker.controller)
}

/// "... it produces twice as much of that mana instead."
fn twice_as_much(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ManaProduction(m) = proposal {
        for n in m.mana.0.iter_mut() {
            *n *= 2;
        }
    }
}

static DOUBLER: CardDef = CardDef {
    name: "Probe Reflection",
    oracle: true,
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::ManaProduction, you_tap_for_mana, twice_as_much, "twice as much instead")],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&DOUBLER])))
}

/// Path tapped once under the doubler adds {W}{W}, both riding its one delayed ability; both pay for Glory Seeker (Human
/// Soldier, like Kytheon): two triggers, two scries of 1, never one scry 2. Each scry bottoms the top card - Mountain,
/// then Island - leaving Forest on top; and the ability is gone with its mana.
const SCRIPT: &str = "[Format]
Commander
[Bot1 Commander]
Kytheon, Hero of Akros
[Bot1 Deck]
Path of Ancestry
Plains
Probe Reflection
Glory Seeker
Plains
Plains
Plains
Plains
Mountain
Island
Forest
Plains x30
[Bot2 Deck]
Island x40
[Actions]
Bot1.Hand.Path of Ancestry.Play
[T3 Bot1.Main1]
Bot1.Hand.Probe Reflection.Cast
Bot1.Pass
Bot2.Pass
Bot1.Battlefield.Path of Ancestry.Activate(Color: White)
? Bot1.Mana = {W}{W}
? Delayed.Count = 1
Bot1.Hand.Glory Seeker.Cast
Bot1.Arrange( | Mountain)
Bot1.Arrange( | Island)
Bot1.Pass
Bot2.Pass
[Expect]
Bot1.Battlefield.Glory Seeker.Power = 2
Bot1.Library.Top = Forest
Delayed.Count = 0
";

#[test]
fn one_activation_doubled_rides_both_mana_and_triggers_once_for_each() {
    let script = GameScript::parse(SCRIPT, "path-of-ancestry-doubled-mana").unwrap();
    if let Err(e) = run_on(&script, table()) {
        panic!("{}", e.report);
    }
}
