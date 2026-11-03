use cantrip_api::*;
use cantrip_mechanics::{lands, regenerate, statics};
use cantrip_oracle as oracle;

fn other_zombie_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Creature) && statics::other_of_subtype::<{ lands::sub(Subtype::Zombie) }>(q, source, affected, chars)
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, other_zombie_creatures, statics::have_landwalk::<{ lands::sub(Subtype::Swamp) }>, "Other Zombie creatures have swampwalk."),
        static_ability(Layer::Ability, statics::other_of_subtype::<{ lands::sub(Subtype::Zombie) }>, statics::grant_delayed_ability, "Other Zombies have \"{B}: Regenerate this permanent.\""),
    ],
    delayed: &[ability(cost("{B}"), regenerate::this_effect, "{B}: Regenerate this permanent.")],
    ..oracle::ZOMBIE_MASTER
};
