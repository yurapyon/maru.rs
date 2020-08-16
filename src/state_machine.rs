pub enum Transition<I, E> {
    Pop,
    Push(Box<dyn State<I, E>>),
    Switch(Box<dyn State<I, E>>),
}

pub struct StateMachine<I, E> {
    states: Vec<Box<dyn State<I, E>>>,
    transition: Option<Transition<I, E>>,
}

// note:
//   doesnt pop to empty
//   no outside way to push a state, eveything has to be handled by states
impl<I, E> StateMachine<I, E> {
    pub fn new(initial_state: Box<dyn State<I, E>>) -> Self {
        Self {
            states: vec![initial_state],
            transition: None,
        }
    }

    pub fn start(&mut self, immut_data: &I) {
        let state = self.states.last_mut().expect("state stack underflow");
        state.on_start(immut_data);
    }

    pub fn maybe_do_transition(&mut self, immut_data: &I) {
        if let Some(tr) = self.transition.take() {
            match tr {
                Transition::Pop        => self.pop(immut_data),
                Transition::Push(st)   => self.push(st, immut_data),
                Transition::Switch(st) => self.switch(st, immut_data),
            }
        }
    }

    pub fn handle_event(&mut self, immut_data: &I, ev: &E) {
        let state = self.states.last_mut().expect("state stack underflow");
        state.handle_event(immut_data, ev);
    }

    pub fn frame(&mut self, immut_data: &I) {
        if let Some((last, elems)) = self.states.split_last_mut() {
            elems.iter_mut().for_each(| s | s.frame_hidden(immut_data));
            self.transition = last.frame(immut_data);
        } else {
            panic!("state stack underflow");
        }
    }

    pub fn fixed_frame(&mut self, immut_data: &I) {
        if let Some((last, elems)) = self.states.split_last_mut() {
            elems.iter_mut().for_each(| s | s.fixed_frame_hidden(immut_data));
            last.fixed_frame(immut_data);
        } else {
            panic!("state stack underflow");
        }
    }

    fn pop(&mut self, immut_data: &I) {
        if self.states.len() == 0 {
            panic!("state stack underflow");
        } else if self.states.len() == 1 {
            panic!("attempting to .pop() last state on stack");
        }

        let mut prev_state = self.states.pop().unwrap();
        prev_state.on_stop(immut_data);

        let next_state = self.states.last_mut().unwrap();
        next_state.on_resume(immut_data);
    }

    fn push(&mut self, next: Box<dyn State<I, E>>, immut_data: &I) {
        let state = self.states.last_mut().expect("state stack underflow");
        state.on_pause(immut_data);

        let mut next = next;
        next.on_start(immut_data);
        self.states.push(next);
    }

    fn switch(&mut self, next: Box<dyn State<I, E>>, immut_data: &I) {
        let mut state = self.states.pop().expect("state stack underflow");
        state.on_stop(immut_data);

        let mut next = next;
        next.on_start(immut_data);
        self.states.push(next);
    }
}

//

pub trait State<I, E> {
    fn on_start(&mut self, _immut_data: &I) {}
    fn on_stop(&mut self, _immut_data: &I) {}
    fn on_pause(&mut self, _immut_data: &I) {}
    fn on_resume(&mut self, _immut_data: &I) {}
    fn handle_event(&mut self, _immut_data: &I, _ev: &E) {}
    fn frame(&mut self, _immut_data: &I) -> Option<Transition<I, E>> {
        None
    }
    fn fixed_frame(&mut self, _immut_data: &I) { }
    fn frame_hidden(&mut self, _immut_data: &I) {}
    fn fixed_frame_hidden(&mut self, _immut_data: &I) {}
}
