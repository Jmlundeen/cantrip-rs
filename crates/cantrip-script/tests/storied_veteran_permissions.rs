use std::sync::OnceLock;

use cantrip_engine::*;
use cantrip_script::{GameScript, run_on};

const SCENE: &str = include_str!(
    "../../../scenarios/storied-veteran-graveyard-permission-stops-when-abilities-are-lost.cantrip"
);
const REFUSAL: &str = "! Bot1.Graveyard.Storied Veteran.Cast\n";

/// "Cards in graveyards lose all abilities" (CR 613.1f, 604.2): Yixlid Jailer's effect.
fn in_a_graveyard(view: &dyn LayerQuery, _: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    view.state().obj(affected).zone.is(ZoneKind::Graveyard)
}

static SILENCE: CardDef = CardDef {
    name: "Probe Graveyard Silence",
    oracle: true,
    has_cost: true,
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[
        loses_all_abilities(in_a_graveyard, "Cards in graveyards lose all abilities.").everywhere(),
    ],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&SILENCE])))
}

#[test]
fn the_own_graveyard_permission_ends_when_the_card_loses_its_abilities() {
    let script = GameScript::parse(SCENE, "storied-veteran-permission-loss").unwrap();
    let result = run_on(&script, table()).unwrap_or_else(|failure| panic!("{}", failure.report));
    let game = &result.game;
    let veteran = game
        .state
        .zone(Zone::graveyard(PlayerId(0)))
        .iter()
        .copied()
        .find(|&id| game.name(id) == "Storied Veteran")
        .expect("Veteran remains in its owner's graveyard");
    assert!(game.lost_all_abilities(veteran));
}

#[test]
fn another_permission_still_casts_the_card_after_its_own_ability_is_lost() {
    let script_text = include_str!("../../../scenarios/storied-veteran-external-permission-still-works-after-ability-loss.cantrip");
    let script = GameScript::parse(script_text, "storied-veteran-external-permission-after-ability-loss").unwrap();
    let result = run_on(&script, table()).unwrap_or_else(|failure| panic!("{}", failure.report));
    let game = &result.game;
    let veteran = game
        .state
        .zone(Zone::new(ZoneKind::Battlefield, PlayerId(0)))
        .iter()
        .copied()
        .find(|&id| game.name(id) == "Storied Veteran")
        .expect("the separate graveyard permission cast the Veteran");
    assert_eq!(game.state.obj(veteran).counters(CounterKind::Finality), 0);
}

#[test]
fn the_intrinsic_permission_is_offered_without_ability_loss_and_its_spell_enters_with_finality() {
    let text = SCENE
        .replace("\r\n", "\n")
        .replace("Probe Graveyard Silence\n", "")
        .replace("Bot1.Hand.Probe Graveyard Silence.Cast\n", "")
        .replace(
            REFUSAL,
            "Bot1.Graveyard.Storied Veteran.Cast\nBot1.Pass\nBot2.Pass\n",
        )
        .replace(
            "[Expect]\nBot1.Graveyard = Innocent Blood; Storied Veteran; Black Lotus\nBot1.Battlefield.Storied Veteran = (absent)\nBot1.Mana = {B}\nExile.Count = 0",
            "[Expect]\nBot1.Graveyard = Innocent Blood; Black Lotus\nBot1.Battlefield.Storied Veteran.Counters.Finality = 1\nExile.Count = 0",
        );
    let script = GameScript::parse(&text, "storied-veteran-permission-positive-control").unwrap();
    let result = run_on(&script, table()).unwrap_or_else(|failure| panic!("{}", failure.report));
    let game = &result.game;
    let veteran = game
        .state
        .zone(Zone::new(ZoneKind::Battlefield, PlayerId(0)))
        .iter()
        .copied()
        .find(|&id| game.name(id) == "Storied Veteran")
        .expect("the graveyard permission cast the Veteran");
    assert_eq!(game.state.obj(veteran).counters(CounterKind::Finality), 1);
}
