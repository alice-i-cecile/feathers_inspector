//! Demonstrates how to modify component values dynamically,
//! using Bevy's reflection and Feathers Inspector.

use core::any::TypeId;

use bevy::{prelude::*, reflect::ReflectMut};
use feathers_inspector::reflection_tools::get_reflected_component_mut;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SelectedComponent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (select_component_to_modify, modify_selected_component),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        image: asset_server.load("ducky.png"),
        ..Default::default()
    });

    let instructions = "\
Press 'T' to select Transform component's y-translation for modification
Press 'S' to select Sprite component's alpha value for modification
Press 'Up Arrow' to increase the selected component's value
Press 'Down Arrow' to decrease the selected component's value"
        .to_string();

    commands.spawn((
        Text::new(instructions),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

// The component type to modify should generally be selected via UI.
#[derive(Resource, Default, Clone)]
enum SelectedComponent {
    #[default]
    Transform,
    Sprite,
}

// Quickly select which component to modify via keyboard input,
// avoiding the need for a full UI in this example.
fn select_component_to_modify(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedComponent>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        *selected = SelectedComponent::Transform;
        info!("Selected Transform component for modification");
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        *selected = SelectedComponent::Sprite;
        info!("Selected Sprite component for modification");
    }
}

// This function demonstrates the core logic of modifying a component value via reflection.
// You must:
// 1. Determine the entity whose component you want to modify.
// 2. Determine the TypeId of the component type to modify.
// 3. Get a mutable reference to the component as `&mut dyn Reflect`.
// 4. Determine the shape of the type using relection, by converting that to a `ReflectMut` object.
// 5. Find the field(s) you want to modify.
// 6. Downcast the value to a concrete type to read existing field values.
// 7. Construct a replacement value based on the existing value.
// 9. Modify the existing value using PartialReflect::apply.
fn modify_selected_component(world: &mut World) {
    // We're using keyboard input to trigger modifications for simplicity.
    let button_input = world.resource::<ButtonInput<KeyCode>>();
    let direction_of_modification = if button_input.pressed(KeyCode::ArrowUp) {
        1.0
    } else if button_input.pressed(KeyCode::ArrowDown) {
        -1.0
    } else {
        return; // No modification requested
    };

    let selected = world.resource::<SelectedComponent>().clone();

    let mut sprite_query = world.query_filtered::<Entity, With<Sprite>>();

    // This entity should generally be gathered via UI selection in a real application
    let entity = sprite_query.iter(&world).next().unwrap();

    // The type id of the component to modify should be gathered via UI selection in a real application
    let type_id = match selected {
        SelectedComponent::Transform => TypeId::of::<Transform>(),
        SelectedComponent::Sprite => TypeId::of::<Sprite>(),
    };

    let mut dynamic_mut = get_reflected_component_mut(world, entity, type_id).unwrap();

    match selected {
        // If we know the type, we can downcast and modify directly.
        SelectedComponent::Sprite => {
            let downcasted = dynamic_mut.downcast_mut::<Sprite>().unwrap();
            // Be careful not to modify a copy of the color!
            let color = &mut downcasted.color;

            let new_alpha = (color.alpha() + 0.01 * direction_of_modification).clamp(0.0, 1.0);
            color.set_alpha(new_alpha);
        }
        // Alternatively, we can walk the reflected type info to find fields to modify.
        SelectedComponent::Transform => {
            let reflect_mut = dynamic_mut.reflect_mut();
            // In the generic case, we would want to match on the `ReflectMut` variants
            let ReflectMut::Struct(struct_mut) = reflect_mut else {
                error!("Expected Transform to be a struct");
                return;
            };

            // Get the `translation` field
            let translation_field = struct_mut.field_mut("translation").unwrap();

            // Now, repeat the process to get the `y` field of the `translation` Vec3
            let ReflectMut::Struct(translation_struct) = translation_field.reflect_mut() else {
                error!("Expected translation to be a struct");
                return;
            };

            let y_field = translation_struct.field_mut("y").unwrap();

            // Check that the field is a primitive type that we know how to handle
            assert!(y_field.get_represented_type_info().unwrap().type_id() == TypeId::of::<f32>());

            // Convert the field to a concrete type to read the current value
            let current_y = y_field.try_downcast_ref::<f32>().unwrap();
            let new_y = current_y + 10.0 * direction_of_modification;

            // Set the new value using reflection
            y_field.try_apply(&new_y).unwrap();
        }
    }
}
