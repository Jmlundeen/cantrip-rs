use cantrip_api::*;
use cantrip_mechanics::{statics, tokens};
use cantrip_oracle as oracle;

/// "Whenever this creature attacks or becomes the target of a spell" (CR 508.1m, 603.2): one triggered ability with two
/// events. A spell, not an ability, naming it as a target.
fn attacks_or_becomes_the_target_of_a_spell(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => Fired::PLAIN,
        Event::BecameTarget { target: Target::Object(t), ability: false, .. } if *t == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Treasures you control have "{T}, Sacrifice this artifact: Add two mana of any one color."" (CR 604.1).
fn add_two_mana_any_one_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    (*so_far).has_subtype(q.words(asker.source).subtype(Subtype::Treasure))
        && asker.controller == q.controller(affected)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(
                EventMask(EventMask::ATTACKS.0 | EventMask::BECAME_TARGET.0),
                attacks_or_becomes_the_target_of_a_spell,
            ),
            tokens::create_a_treasure,
            "Whenever this creature attacks or becomes the target of a spell, create a Treasure token.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            add_two_mana_any_one_affects,
            statics::grant_delayed_ability,
            "Treasures you control have \"{T}, Sacrifice this artifact: Add two mana of any one color.\"",
        ),
    ],
    delayed: &[
        mana_ability(
            SACRIFICE_THIS.and(TAP),
            add_any_color(2),
            "{T}, Sacrifice this artifact: Add two mana of any one color.",
        ),
    ],
    ..oracle::GOLDSPAN_DRAGON
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
