use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SpritePool {
    pub entities: Vec<Entity>,
    pub used: usize,
}

impl SpritePool {
    pub fn next(&mut self, commands: &mut Commands) -> Entity {
        if self.used >= self.entities.len() {
            let e = commands
                .spawn((Sprite::default(), Transform::default(), Visibility::Hidden))
                .id();
            self.entities.push(e);
        }
        let e = self.entities[self.used];
        self.used += 1;
        e
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }

    pub fn hide_unused(&self, query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>) {
        for &e in &self.entities[self.used..] {
            if let Ok((_, _, mut vis)) = query.get_mut(e) {
                *vis = Visibility::Hidden;
            }
        }
    }
}
