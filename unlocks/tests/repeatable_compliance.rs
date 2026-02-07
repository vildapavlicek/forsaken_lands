use {bevy::prelude::*, unlocks::*};

#[test]
fn tested_repeatable_modes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(UnlocksPlugin)
        .add_plugins(AssetPlugin::default())
        .init_asset::<UnlockDefinition>();

    // 1. Setup Assets
    let mut handles = Vec::new();
    let mut assets = app.world_mut().resource_mut::<Assets<UnlockDefinition>>();

    // Finite(2) Unlock
    handles.push(assets.add(UnlockDefinition {
        id: "finite_test".to_string(),
        display_name: None,
        condition: ConditionNode::Completed {
            topic: "test:finite".to_string(),
        },
        reward_id: "reward:finite".to_string(),
        repeat_mode: RepeatMode::Finite(2),
    }));

    // Infinite Unlock
    handles.push(assets.add(UnlockDefinition {
        id: "infinite_test".to_string(),
        display_name: None,
        condition: ConditionNode::Completed {
            topic: "test:infinite".to_string(),
        },
        reward_id: "reward:infinite".to_string(),
        repeat_mode: RepeatMode::Infinite,
    }));

    // Once Unlock
    handles.push(assets.add(UnlockDefinition {
        id: "once_test".to_string(),
        display_name: None,
        condition: ConditionNode::Completed {
            topic: "test:once".to_string(),
        },
        reward_id: "reward:once".to_string(),
        repeat_mode: RepeatMode::Once,
    }));

    // Force compilation by simulating what LoadingManager does
    // We can't easily wait for assets to load in test without async, so we'll just manually call compile logic
    // actually, let's just use the system `compile_pending_unlocks`.
    // But we need to make sure assets are "added". `Assets` resource is present.
    // The `compile_pending_unlocks` iterates over `Assets<UnlockDefinition>`.

    app.update(); // Initialize plugins

    // Run compile system once to spawn graphs
    app.update();
    // We need to make sure `compile_pending_unlocks` runs. It is not added to Update schedule in Plugin?
    // Let's check `UnlocksPlugin`.
    // Ah, `compile_pending_unlocks` is NOT added to `UnlocksPlugin`. It is called by game code.
    // So we must manually run it or add it to schedule for test.
    app.add_systems(Update, compile_pending_unlocks);
    app.update();

    // Verify graphs are spawned
    assert_graph_exists(&mut app, "finite_test", true);
    assert_graph_exists(&mut app, "infinite_test", true);
    assert_graph_exists(&mut app, "once_test", true);

    // --- TEST FINITE (2) ---

    // Trigger 1
    trigger_completion(&mut app, "test:finite");
    app.update(); // propagate signal -> unlock achieved -> handle lifecycle

    // Should be reset, not despawned
    assert_graph_exists(&mut app, "finite_test", true);
    assert_progress(&app, "finite_test", 1);

    // Trigger 2
    trigger_completion(&mut app, "test:finite");
    app.update();

    // Should be despawned now (max reached)
    assert_graph_exists(&mut app, "finite_test", false);
    assert_progress(&app, "finite_test", 2);

    // Trigger 3 (should have no effect)
    trigger_completion(&mut app, "test:finite");
    app.update();
    assert_progress(&app, "finite_test", 2); // Count shouldn't increase as graph is gone

    // --- TEST INFINITE ---

    // Trigger 1
    trigger_completion(&mut app, "test:infinite");
    app.update();
    assert_graph_exists(&mut app, "infinite_test", true);
    assert_progress(&app, "infinite_test", 1);

    // Trigger 2
    trigger_completion(&mut app, "test:infinite");
    app.update();
    assert_graph_exists(&mut app, "infinite_test", true);
    assert_progress(&app, "infinite_test", 2);

    // --- TEST ONCE ---

    // Trigger 1
    trigger_completion(&mut app, "test:once");
    app.update();
    assert_graph_exists(&mut app, "once_test", false);
    assert_progress(&app, "once_test", 1);

    // Check UnlockState for Once
    let state = app.world().resource::<UnlockState>();
    assert!(state.is_unlocked("once_test"));
}

fn trigger_completion(app: &mut App, topic: &str) {
    app.world_mut().trigger(StatusCompleted {
        topic: topic.to_string(),
    });
}

fn assert_graph_exists(app: &mut App, id: &str, exists: bool) {
    let mut found = false;
    let mut query = app.world_mut().query::<&UnlockRoot>();
    for root in query.iter(app.world()) {
        if root.id == id {
            found = true;
            break;
        }
    }
    assert_eq!(found, exists, "Graph existence mismatch for {}", id);
}

fn assert_progress(app: &App, id: &str, expected: u32) {
    let progress = app.world().resource::<UnlockProgress>();
    let count = *progress.counts.get(id).unwrap_or(&0);
    assert_eq!(count, expected, "Progress count mismatch for {}", id);
}
