use super::DeltaTimer;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct VideoProfile{
	pub width:     usize, /* Render width     */
	pub height:    usize, /* Render height    */
	pub framerate: usize, /* Target framerate */

	pub fullscreen: bool,
	pub vsync: bool
}

/* Type alias to avoid linting errors until I do the *actual* Scene trait */
use glium::Surface;
pub trait Scene{
	fn load(&mut self,      _: &mut Game) { }
	fn pause(&mut self,     _: &mut Game) { }
	fn unpause(&mut self,   _: &mut Game) { }
	fn update(&mut self,    _: &mut Game, _: f64) { }
	fn render(&mut self, game: &mut Game) { game.framebuffer().clear_color(0.0, 0.0, 0.0, 1.0); }
	fn unload(&mut self,    _: &mut Game) { }
}

use super::resource::Texture;
enum ErrorScene{
	ThatsAllFolks(Texture)
}
impl Scene for ErrorScene{
	fn load(&mut self, game: &mut Game){
		match self{
			&mut ErrorScene::ThatsAllFolks(ref mut texture) => {
				*texture = Texture::open(game, "res/fixme.png").unwrap()
			}
		}
	}

	fn render(&mut self, game: &mut Game){

	}
}

enum State  { Running(Box<Scene>), Paused(Box<Scene>), Available, Dead }
impl State{
	fn into_u8(&self) -> u8 {
		match *self{
			State::Running(_) => 0,
			State::Paused(_)  => 1,
			State::Available  => 2,
			State::Dead       => 3
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Target { Continue, Pause, Return, Quit, None }

use std::collections::HashMap;
use std::any::Any;

use glium::Frame;
use glium::backend::glutin_backend::GlutinFacade;
pub struct Game{
	name:  String,
	video: VideoProfile,  /* Video profile the game will be targetting */
	window: GlutinFacade, /* Window being used for input and output    */

	// == Runtime variables == //
	flags: HashMap<String, Box<Any>>, /* Game-wide flags */
	scene_queue: Vec<Box<Scene>>,     /* Buffer of scenes to succeed the current one once it finishes */
	framebuffer: Option<Frame>,       /* Access to the default framebuffer */

	target: Target /* Target execution state */
}
impl Game{
	pub fn new(name: String, video: VideoProfile) -> Game{
		// Build the Window the game will be using
		use glium::DisplayBuild;
		use glium::glutin;
		let mut win = glutin::WindowBuilder::new()
			.with_title(name.clone());

		if video.fullscreen { win = win.with_fullscreen(glutin::get_primary_monitor()) }
		if video.vsync { win = win.with_vsync() }

		Game{
			name: name,
			video: video,
			window: win.build_glium().expect("Could not create main game window!"),


			flags: HashMap::new(),
			scene_queue: Vec::new(),
			framebuffer: None,
			target: Target::None
		}
	}


	/**
	 * Gets the current game framebuffer.
	 *
	 * NOTE: Do NOT call this function outside render(),
	 * doing so will almost certainly cause the program to crash
	 */
	pub fn framebuffer(&mut self) -> &mut Frame{
		self.framebuffer.as_mut().expect(
			"Tried to retrieve the Game's framebuffer while it's not ready to be used.
			 Please make sure you're not trying to get the framebuffer while outside of render()."
		)
	}

	pub fn queue_scene(&mut self, scene: Box<Scene>){
		self.scene_queue.push(scene);
	}

	/* Target controls */
	pub fn quit(&mut self)  { self.target = Target::Quit;   }
	pub fn finish(&mut self){ self.target = Target::Return; }
	pub fn pause(&mut self) {
		if self.target != Target::Quit && self.target != Target::Return {
			self.target = Target::Pause;
		}
	}
	pub fn unpause(&mut self) {
		if let Target::Pause = self.target { self.target = Target::Continue }
	}


}

/* Implement Facade for Game for convenience */
use std::rc::Rc;
use glium::backend::{Facade, Context};
impl Facade for Game{
	fn get_context(&self) -> &Rc<Context>{ self.window.get_context() }
}

pub struct Runner{
	game: Game,

	state:  State,      /* Current state of the game's execution */
	timer:  DeltaTimer  /* For the game's delta time calculation */
}
impl Runner{
	pub fn new(game: Game) -> Runner{
		Runner{
			game: game,
			state: State::Available,
			timer: DeltaTimer::new()
		}
	}

	#[must_use]
	pub fn run(mut self) -> Game{
		while self.state.into_u8() != State::Dead.into_u8() {
			while let State::Running(mut scene) = self.state{
				// Get the delta time for the current frame
				let delta = self.timer.delta();
				scene.update(&mut self.game, delta);

				// Create a new frame, draw and dicard frame
				self.game.framebuffer = Some(self.game.window.draw());
				scene.render(&mut self.game);

				let _ = self.game.framebuffer.unwrap().finish();
				self.game.framebuffer = None;

				// Proccess the game's target
				match self.game.target{
					Target::Pause    => { scene.pause(&mut self.game);  self.state = State::Paused(scene) },
					Target::Return   => { scene.unload(&mut self.game); self.state = State::Available },
					Target::Quit     => { scene.unload(&mut self.game); self.state = State::Dead },

					/* Otherwise, continue running the state */
					_ => { self.state = State::Running(scene) }
				}

				// And reset it
				self.game.target = Target::None;
			}

			self.state = match self.state {
				State::Paused(mut scene) => {
					if let Target::Continue = self.game.target {
						scene.unpause(&mut self.game);
						State::Running(scene)
					}else{
						/* TODO: Proccess external events */
						State::Paused(scene)
					}
				},
				State::Available => {
					/* Load and start running the next scene from the game queue */
					let mut scene = if self.game.scene_queue.len() > 0 {
						println!("Found scene!");
						self.game.scene_queue.remove(0)
					}else{ Box::new(ErrorScene::ThatsAllFolks(Texture::None)) as Box<Scene> };

					scene.load(&mut self.game);
					State::Running(scene)
				},

				/* Ignore other states */
				state @ _ => { state }
			}
		}

		/* Return game once finished running */
		self.game
	}
}

#[test]
fn scene(){
	struct Sc0{ count: f64, color: (f32, f32, f32) }
	impl Scene for Sc0{
		fn load(&mut self, _: &mut Game){ self.count = 0.0; }
		fn update(&mut self, g: &mut Game, d: f64){
			if self.count >= 2.3{ g.quit(); }
			self.count += d;

			self.color = (
				0.0,
				self.count as f32 / 2.3,
				self.count as f32 / 2.3,
			);

			println!("[Sc] ( count: {}, color: {:?} )", self.count, self.color);
		}
		fn render(&mut self, g: &mut Game){
			use glium::Surface;
			g.framebuffer().unwrap().clear_color(self.color.0, self.color.1, self.color.2, 1.0);
		}
	}
	struct Sc1{ count: f64, color: (f32, f32, f32) }
	impl Scene for Sc1{
		fn load(&mut self, _: &mut Game){ self.count = 0.0; }
		fn update(&mut self, g: &mut Game, d: f64){
			if self.count >= 2.3{
				g.queue_scene(Box::new(Sc0{ count: 0.0, color: (0.0, 0.0, 0.0) }));
				g.finish();
			}
			self.count += d;

			self.color = (
				0.0,
				1.0 - if self.count as f32 / 2.3 > 1.0 { 1.0 } else { self.count as f32 / 2.3 },
				1.0 - if self.count as f32 / 2.3 > 1.0 { 1.0 } else { self.count as f32 / 2.3 },
			);

			println!("[Sc] ( count: {}, color: {:?} )", self.count, self.color);
		}
		fn render(&mut self, g: &mut Game){
			use glium::Surface;
			g.framebuffer().unwrap().clear_color(self.color.0, self.color.1, self.color.2, 1.0);
		}
	}

	let profile = VideoProfile{
		width: 800,
		height: 600,
		framerate: 60,
		fullscreen: false,
		vsync: true
	};
	let mut game = Game::new("Automated test: scene()".to_owned(), profile);
	game.queue_scene(Box::new(Sc1{ count: 0.0, color: (0.0, 0.0, 0.0) }));

	// Run and dispose of the game
	let _ = Runner::new(game).run();
}
