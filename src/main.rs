use std::io::{self, stdout};
use std::time::Duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
    Frame, Terminal,
};

// The visualizer state that tracks comparisons and swaps
#[derive(Clone)]
pub struct VisualizerState {
    pub array: Vec<u32>,
    pub comparing: Option<(usize, usize)>,
    pub swapping: Option<(usize, usize)>,
    pub sorted: Vec<bool>,
    pub comparisons: usize,
    pub swaps: usize,
}

impl VisualizerState {
    pub fn new(array: Vec<u32>) -> Self {
        let len = array.len();
        Self {
            array,
            comparing: None,
            swapping: None,
            sorted: vec![false; len],
            comparisons: 0,
            swaps: 0,
        }
    }

    pub fn mark_comparing(&mut self, i: usize, j: usize) {
        self.comparing = Some((i, j));
        self.comparisons += 1;
    }

    pub fn mark_swapping(&mut self, i: usize, j: usize) {
        self.swapping = Some((i, j));
        self.swaps += 1;
    }

    pub fn clear_marks(&mut self) {
        self.comparing = None;
        self.swapping = None;
    }

    pub fn mark_sorted(&mut self, indices: &[usize]) {
        for &i in indices {
            if i < self.sorted.len() {
                self.sorted[i] = true;
            }
        }
    }
}

// Trait that your sorting algorithms should implement
pub trait SortingAlgorithm {
    fn name(&self) -> &str;
    fn sort(&self, state: &mut VisualizerState, step_callback: &mut dyn FnMut(&VisualizerState));
}

// Bubble Sort
pub struct BubbleSort;

impl SortingAlgorithm for BubbleSort {
    fn name(&self) -> &str {
        "Bubble Sort"
    }

    fn sort(&self, state: &mut VisualizerState, step_callback: &mut dyn FnMut(&VisualizerState)) {
        let n = state.array.len();
        for i in 0..n {
            for j in 0..n - i - 1 {
                state.mark_comparing(j, j + 1);
                step_callback(state);
                
                if state.array[j] > state.array[j + 1] {
                    state.mark_swapping(j, j + 1);
                    step_callback(state);
                    state.array.swap(j, j + 1);
                }
                
                state.clear_marks();
            }
            state.mark_sorted(&[n - i - 1]);
            step_callback(state);
        }
    }
}

// Quick Sort (with insertion sort for small partitions)
pub struct QuickSort;

impl SortingAlgorithm for QuickSort {
    fn name(&self) -> &str {
        "Quick Sort (+ Insertion)"
    }
    
    fn sort(&self, state: &mut VisualizerState, step_callback: &mut dyn FnMut(&VisualizerState)) {
        let high = state.array.len() - 1;
        quicksort_helper(state, 0, high, step_callback);
        for i in 0..high + 1 {
            state.mark_sorted(&[i]);
            step_callback(state);
        }
    }
}

const INSERTION_THRESHOLD: usize = 10;

fn quicksort_helper(
    state: &mut VisualizerState, 
    low: usize, 
    high: usize, 
    step_callback: &mut dyn FnMut(&VisualizerState)
) {
    if low < high {
        if high - low < INSERTION_THRESHOLD {
            insertion_sort_range(state, low, high, step_callback);
            return;
        }
        
        let pi = partition(state, low, high, step_callback);
        
        if pi > 0 {
            quicksort_helper(state, low, pi - 1, step_callback);
        }
        quicksort_helper(state, pi + 1, high, step_callback);
    }
}

fn insertion_sort_range(
    state: &mut VisualizerState,
    low: usize,
    high: usize,
    step_callback: &mut dyn FnMut(&VisualizerState)
) {
    for i in (low + 1)..=high {
        let key = state.array[i];
        let mut j = i as isize - 1;
        
        while j >= low as isize && state.array[j as usize] > key {
            state.mark_comparing(j as usize, (j + 1) as usize);
            step_callback(state);
            
            state.mark_swapping(j as usize, (j + 1) as usize);
            step_callback(state);
            
            state.array[(j + 1) as usize] = state.array[j as usize];
            j -= 1;
        }
        
        state.array[(j + 1) as usize] = key;
        state.clear_marks();
        step_callback(state);
    }
    for i in low..high + 1 {
        state.mark_sorted(&[i]);
        step_callback(state);
    }
}

fn partition(
    state: &mut VisualizerState, 
    low: usize, 
    high: usize, 
    step_callback: &mut dyn FnMut(&VisualizerState)
) -> usize {
    let pivot = state.array[high];
    let mut i = low as isize - 1;
    
    for j in low..high {
        state.mark_comparing(j, high);
        step_callback(state);
        
        if state.array[j] <= pivot {
            i += 1;
            state.mark_swapping(i as usize, j);
            step_callback(state);
            state.array.swap(i as usize, j);
        }
        state.clear_marks();
    }
    
    state.mark_swapping((i + 1) as usize, high);
    step_callback(state);
    state.array.swap((i + 1) as usize, high);
    state.clear_marks();
    
    state.mark_sorted(&[(i + 1) as usize]);
    
    (i + 1) as usize
}

// Insertion Sort
pub struct InsertionSort;

impl SortingAlgorithm for InsertionSort {
    fn name(&self) -> &str {
        "Insertion Sort"
    }
    
    fn sort(&self, state: &mut VisualizerState, step_callback: &mut dyn FnMut(&VisualizerState)) {
        let n = state.array.len();
        for i in 1..n {
            let key = state.array[i];
            let mut j = i as isize - 1;
            
            while j >= 0 && state.array[j as usize] > key {
                state.mark_comparing(j as usize, (j + 1) as usize);
                step_callback(state);
                
                state.mark_swapping(j as usize, (j + 1) as usize);
                step_callback(state);
                
                state.array[(j + 1) as usize] = state.array[j as usize];
                j -= 1;
            }
            
            state.array[(j + 1) as usize] = key;
            state.clear_marks();
            step_callback(state);
        }
        for i in 0..state.array.len() {
            state.mark_sorted(&[i]);
            step_callback(state);
        }
    }
}

// Selection Sort
pub struct SelectionSort;

impl SortingAlgorithm for SelectionSort {
    fn name(&self) -> &str {
        "Selection Sort"
    }
    
    fn sort(&self, state: &mut VisualizerState, step_callback: &mut dyn FnMut(&VisualizerState)) {
        let n = state.array.len();
        
        for i in 0..n - 1 {
            let mut min_idx = i;
            
            for j in (i + 1)..n {
                state.mark_comparing(j, min_idx);
                step_callback(state);
                
                if state.array[j] < state.array[min_idx] {
                    min_idx = j;
                }
                state.clear_marks();
            }
            
            if min_idx != i {
                state.mark_swapping(i, min_idx);
                step_callback(state);
                state.array.swap(i, min_idx);
            }
            
            state.clear_marks();
            state.mark_sorted(&[i]);
        }
        
        state.mark_sorted(&[n - 1]);
        step_callback(state);
    }
}

// Radix Sort
pub struct RadixSort;

impl SortingAlgorithm for RadixSort {
    fn name(&self) -> &str {
        "Radix Sort"
    }

    fn sort(&self, state: &mut VisualizerState, step_callback: &mut dyn FnMut(&VisualizerState)) {
        let max = *state.array.iter().max().unwrap();
        let mut exp = 1;
        
        while max / exp > 0 {
            counting_sort_by_digit(state, exp, step_callback);
            exp *= 10;
        }
        
        let n = state.array.len();
        for i in 0..n {
            state.mark_sorted(&[i]);
            step_callback(state);
        }
    }
}

fn counting_sort_by_digit(
    state: &mut VisualizerState, 
    exp: u32, 
    step_callback: &mut dyn FnMut(&VisualizerState)
) {
    let n = state.array.len();
    let mut output = vec![0u32; n];
    let mut count = [0usize; 10];
    
    for i in 0..n {
        let digit = ((state.array[i] / exp) % 10) as usize;
        count[digit] += 1;
        state.mark_comparing(i, i);
        step_callback(state);
        state.clear_marks();
    }
    
    for i in 1..10 {
        count[i] += count[i - 1];
    }
    
    for i in (0..n).rev() {
        let digit = ((state.array[i] / exp) % 10) as usize;
        let pos = count[digit] - 1;
        output[pos] = state.array[i];
        count[digit] -= 1;
        
        state.mark_swapping(i, pos);
        step_callback(state);
        state.clear_marks();
    }
    
    for i in 0..n {
        state.array[i] = output[i];
        step_callback(state);
    }
}

struct App {
    state: VisualizerState,
    algorithms: Vec<Box<dyn SortingAlgorithm>>,
    current_algorithm: usize,
    is_sorting: bool,
    is_paused: bool,
    speed: u64,
    steps: Vec<VisualizerState>,
    current_step: usize,
    array_size: usize,
    ludicrous_mode: bool,
    ocd_mode: bool,
}

impl App {
    fn new(array_size: usize) -> Self {
        let algorithms: Vec<Box<dyn SortingAlgorithm>> = vec![
            Box::new(BubbleSort),
            Box::new(QuickSort),
            Box::new(RadixSort),
            Box::new(InsertionSort),
            Box::new(SelectionSort),
        ];
        
        let mut array: Vec<u32> = (1..=array_size as u32).collect();
        array.shuffle(&mut thread_rng());
        
        Self {
            state: VisualizerState::new(array),
            algorithms,
            current_algorithm: 0,
            is_sorting: false,
            is_paused: false,
            speed: 10,  // Start at 10ms instead of 50ms
            steps: Vec::new(),
            current_step: 0,
            array_size,
            ludicrous_mode: false,
            ocd_mode: false,
        }
    }

    fn reset(&mut self) {
        let mut array: Vec<u32> = (1..=self.array_size as u32).collect();
        array.shuffle(&mut thread_rng());
        self.state = VisualizerState::new(array);
        self.is_sorting = false;
        self.is_paused = false;
        self.steps.clear();
        self.current_step = 0;
    }

    fn start_sorting(&mut self) {
        let mut state = self.state.clone();
        self.steps.clear();
        self.steps.push(state.clone());
        
        self.algorithms[self.current_algorithm].sort(&mut state, &mut |s| {
            self.steps.push(s.clone());
        });
        
        self.current_step = 0;
        self.is_sorting = true;
    }

    fn step_forward(&mut self) -> bool {
        if self.ludicrous_mode {
            // LUDICROUS SPEED: Skip 100 steps at once!
            let skip = 100;
            if self.current_step + skip < self.steps.len() - 1 {
                self.current_step += skip;
                self.state = self.steps[self.current_step].clone();
                true
            } else {
                self.current_step = self.steps.len() - 1;
                self.state = self.steps[self.current_step].clone();
                self.is_sorting = false;
                false
            }
        } else if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            self.state = self.steps[self.current_step].clone();
            true
        } else {
            self.is_sorting = false;
            false
        }
    }

    fn next_algorithm(&mut self) {
        if !self.is_sorting {
            self.current_algorithm = (self.current_algorithm + 1) % self.algorithms.len();
            self.reset();
        }
    }

    fn prev_algorithm(&mut self) {
        if !self.is_sorting {
            self.current_algorithm = if self.current_algorithm == 0 {
                self.algorithms.len() - 1
            } else {
                self.current_algorithm - 1
            };
            self.reset();
        }
    }

    fn increase_size(&mut self) {
        if !self.is_sorting && self.array_size < 1000 {
            let increment = if self.array_size < 100 {
                5
            } else if self.array_size < 500 {
                25
            } else {
                50
            };
            self.array_size = (self.array_size + increment).min(1000);
            self.reset();
        }
    }

    fn decrease_size(&mut self) {
        if !self.is_sorting && self.array_size > 5 {
            let decrement = if self.array_size <= 100 {
                5
            } else if self.array_size <= 500 {
                25
            } else {
                50
            };
            self.array_size = self.array_size.saturating_sub(decrement).max(5);
            self.reset();
        }
    }
}

fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Sorting Visualizer - ", Style::default().fg(Color::Cyan)),
        Span::styled(
            app.algorithms[app.current_algorithm].name(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Bar chart
    let max_val = *app.state.array.iter().max().unwrap_or(&1) as u64;
    
    let mut bar_data = Vec::new();
    for (i, &val) in app.state.array.iter().enumerate() {
        let color = if app.state.sorted[i] {
            Color::Green
        } else if let Some((a, b)) = app.state.swapping {
            if i == a || i == b {
                Color::Red
            } else {
                Color::Blue
            }
        } else if let Some((a, b)) = app.state.comparing {
            if i == a || i == b {
                Color::Yellow
            } else {
                Color::Blue
            }
        } else {
            Color::Blue
        };
        
        bar_data.push(Bar::default().style(Style::default().fg(color)).value(val as u64).text_value(if app.ocd_mode { "".to_string() } else { val.to_string() }));
    }

    // Calculate bar width based on available space
    let available_width = chunks[1].width.saturating_sub(2); // Account for borders
    let total_bars = app.state.array.len();
    
    // Allow bars to be very thin for large arrays
    let bar_width = if total_bars > available_width as usize {
        1 // Minimum width
    } else {
        ((available_width as usize / total_bars).max(1).min(5)) as u16
    };
    let bar_gap = if bar_width > 1 { 1 } else { 0 };

    let barchart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title("Array Visualization"))
        .data(BarGroup::default().bars(&bar_data))
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .max(max_val);
    
    f.render_widget(barchart, chunks[1]);

    // Controls and stats
    let status = if app.is_sorting {
        if app.is_paused {
            "PAUSED"
        } else if app.ludicrous_mode {
            "🚀 LUDICROUS SPEED 🚀"
        } else {
            "SORTING"
        }
    } else {
        "READY"
    };

    let status_color = if app.ludicrous_mode {
        Color::Magenta
    } else {
        Color::Yellow
    };

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            Span::styled(status, Style::default().fg(status_color)),
            Span::raw("  |  "),
            Span::styled("Array Size: ", Style::default().fg(Color::Cyan)),
            Span::raw(app.array_size.to_string()),
            Span::raw("  |  "),
            Span::styled("Comparisons: ", Style::default().fg(Color::Cyan)),
            Span::raw(app.state.comparisons.to_string()),
            Span::raw("  |  "),
            Span::styled("Swaps: ", Style::default().fg(Color::Cyan)),
            Span::raw(app.state.swaps.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("[SPACE] Start/Pause  [R] Reset  [←/→] Change Algorithm  [↑/↓] Array Size  [+/-] Speed  [L] Ludicrous  [O] Ocd Mode [Q] Quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Info"));
    f.render_widget(info, chunks[2]);
}

fn main() -> io::Result<()> {
    let mut app = App::new(30); // Start with 30 elements

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw_ui(f, &app))?;

        if event::poll(Duration::from_millis(app.speed))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        if !app.is_sorting {
                            app.start_sorting();
                        } else {
                            app.is_paused = !app.is_paused;
                        }
                    }
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Left => app.prev_algorithm(),
                    KeyCode::Right => app.next_algorithm(),
                    KeyCode::Up => app.increase_size(),
                    KeyCode::Down => app.decrease_size(),
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        if app.speed > 1 {
                            app.speed = (app.speed - 1).max(1);
                        }
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        app.speed = (app.speed + 1).min(100);
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        app.ludicrous_mode = !app.ludicrous_mode;
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        app.ocd_mode = !app.ocd_mode
                    }
                    _ => {}
                }
            }
        } else if app.is_sorting && !app.is_paused {
            app.step_forward();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
