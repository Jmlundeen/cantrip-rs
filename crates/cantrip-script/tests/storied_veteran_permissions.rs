use std::sync::OnceLock;

use cantrip_engine::*;
use cantrip_script::{GameScript, run_on};

const SCENE: &str = include_str!(
    "../../../scenarios/storied-veteran-graveyard-permission-stops-when-abilities-are-lost.cantrip"
);
const REFUSAL: &str = "! Bot1.Graveyard.Storied Veteran.Cast\n";

// The probe tests only casting permission; it is not registered as a functional Storied Veteran.
static VETERAN: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBlock, Keyword::CastFromYourGraveyard]),
    ..cantrip_oracle::STORIED_VETERAN
};

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
    TABLE.get_or_init(|| {
        CardTable::new(cantrip_cards::all().into_iter().chain([&VETERAN, &SILENCE]))
    })
}

#[test]
fn the_intrinsic_permission_loss_regression_is_still_failing() {
    let outcome = std::panic::catch_unwind(|| {
        let script = GameScript::parse(SCENE, "storied-veteran-permission-loss").unwrap();
        run_on(&script, table())
    });
    match outcome {
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or_default();
            assert_eq!(
                message,
                "Probe Graveyard Silence: a static ability that makes objects lose all abilities affects permanents"
            );
            println!("known bug: {message}");
        }
        Ok(Err(failure)) => {
            assert!(
                failure.message.contains("must not be offered, but it is"),
                "{}",
                failure.report
            );
            assert!(
                failure
                    .message
                    .contains("Bot1.Graveyard.Storied Veteran.Cast"),
                "{}",
                failure.report
            );
            println!("known bug: {}", failure.message);
        }
        Ok(Ok(_)) => panic!("the known bug is fixed; remove its marker and this guard"),
    }
}

#[test]
fn the_same_graveyard_position_offers_the_intrinsic_permission_without_ability_loss() {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    let table =
        TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&VETERAN])));
    let text = SCENE
        .replace("Probe Graveyard Silence\n", "")
        .replace("Bot1.Hand.Probe Graveyard Silence.Cast\n", "")
        .replace(REFUSAL, "");
    let script = GameScript::parse(&text, "storied-veteran-permission-positive-control").unwrap();
    let result = run_on(&script, table).unwrap_or_else(|failure| panic!("{}", failure.report));
    let game = &result.game;
    let veteran = game
        .state
        .zone(Zone::graveyard(PlayerId(0)))
        .iter()
        .copied()
        .find(|&id| game.name(id) == VETERAN.name)
        .unwrap();
    assert!(!game.lost_all_abilities(veteran));
    assert!(game.chars(veteran).has(Keyword::CastFromYourGraveyard));
    let RequestKind::Priority { options } = &game.pending().unwrap().kind else {
        panic!("priority")
    };
    assert!(
        options
            .iter()
            .any(|option| option.kind == ActionKind::CastSpell && option.source == Some(veteran))
    );
}
