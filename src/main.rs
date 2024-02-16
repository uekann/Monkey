use Monkey::repl;

const MONKEY_FACE: &str = r#"
            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"`` ``"-./, -' /
   `'-' /_   ^ ^   _\ '-'`
       |  \._   _./  |
       \   \ `~` /   /
        '._ '---' _.'
           '~---~'

"#;

fn main() {
    println!("{}", MONKEY_FACE);
    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");

    repl::start();
    println!("Goodbye!")
}
