use {bevy::prelude::*, core::CorePlugin, hero_ui::HeroUiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(HeroUiPlugin)
        .run();
}
