Lab number: CSE 5402 Fall 2025 Lab 2
Zichu Pan p.zichu@wustl.edu
Edgar Palomino e.j.palomino@wustl.edu

Structs:
    Player Struct:
        Data: 
            character name, 
            lines (as tuples of line number and text)
            current line index

        Methods:
            new(): Constructor that creates a new player with a given name
            prepare(): Reads a part file, parses line numbers, and sorts lines
                add_script_line(): Private helper that parses individual lines
            speak(): Outputs the next line and updates the current speaker
            next_line(): Returns the line number of the next unspoken line (or None)
    
        Play Struct:
            Data: 
                play title
                vector of Player structs

            Methods:
                new(): Constructor that creates an empty play
                prepare(): Main entry point that reads config file and initializes all players
                read_config(): Parses the configuration file and extracts title and character info
                    dd_config(): Private helper that parses individual config lines
                process_config(): Creates and prepares Player instances from config data
                recite(): Orchestrates the performance by having players speak in line number order
    
    Design Challenges:
        The play needs to deliver lines in global line number order, but each player only knows their own lines. To solve this
        the recite() method iterates through all players each turn, finds the player with the smallest next line number using 
        next_line(), and has that player speak(). This continues until all players have exhausted their lines.

        The program also needs to detect and warn about missing or duplicate line numbers in whinge mode. To solve this
        the recite() method maintains an expected_line_number counter that tracks what line should come next. By comparing 
        each actual line number (note: actual line number is always the smallest line number from all possible players) to the 
        expected value: 
            If actual > expected: missing lines detected (warn about each gap)
            If actual == expected: normal case (increment expected)
            If actual < expected: duplicate detected (we've seen this number before)
    
Return Wrapper:
    Data:
        A single code: u8 field that stores the exit code value

    The Termination trait implementation handles the conversion from our custom type to a shell exit code. The report() method:
        1. Checks if the code is non-zero (indicating an error)
        2. If non-zero, prints a formatted error message to stderr: "Error: {code}"
        3. Converts the u8 code to an ExitCode using ExitCode::from()
        4. Returns the ExitCode for the shell environment

    The main function was modified to change return type from Result<(), u8> to ReturnWrapper and wrap all return values 
    with ReturnWrapper::new():
        ReturnWrapper::new(0) for successful execution
        ReturnWrapper::new(BAD_COMMAND_LINE_ERROR) for command-line argument errors
        ReturnWrapper::new(SCRIPT_GENERATION_ERROR) for file I/O or parsing errors
    
    No design Challenges encountered.

Scene Fragments:
    SceneFragment Struct:
        Data:
            scene title (String)
            vector of Player structs
        Methods:
            new(): Constructor that creates a new scene fragment with a given title
            prepare(): Main setup that reads scene config file and initializes all players for this scene
                read_config(): Parses the configuration file mapping character names to script files
                add_config(): Private helper that parses individual config lines (name + filename pairs)
                process_config(): Creates and prepares Player instances from config data
            has_title(): Returns whether the scene has a non-empty title
            print_title(): Displays the scene title
            enter_all(): Has all players enter (used for first scene only)
            enter(): Has only new players enter (compares with previous scene's player list)
            exit(): Has departing players exit in reverse order (compares with next scene's player list)
            exit_all(): Has all players exit in reverse order (used for final scene only)
            recite(): Orchestrates dialogue delivery by having players speak in line number order

    Design Challenges:
        Players may appear in multiple consecutive scenes. The code needs to avoid having 
        players exit and immediately re-enter between scenes. Implemented enter() and exit() methods 
        that compare player lists between consecutive scenes. Only players new to a scene announce entrance, 
        and only players not continuing to the next scene announce exit. The first scene uses enter_all() 
        and the final scene uses exit_all().

Testing:
    14 test cases are used and all passed with expected behavior. See /test directory for all test cases
    Note: all test cases at least generate 1 warning message "line 0 missing" for testing
        Good case:
            Test 0: example provided
            Test 1: Simple two-scene play with different characters
            Test 11: Three scenes demonstrating player continuity
            Test 12: Lines in file are out of order but should be sorted correctly
            Test 13: One player (Oliver) has no lines

        Error case (fail with error code):
            Test 2: First scene fragment has no title
                Exit code: 3 (SCRIPT_PARSING_ERROR)
            Test 3: Script file is completely empty
                Exit code: 3 (SCRIPT_PARSING_ERROR)
            Test 10: Scene config file is empty
                Exit code: 4 (CONFIG_PARSING_ERROR)


        Warning case (succeed but generated warning in whinge mode):
            Test 4: Player has duplicate line number 2
            Test 5: Missing line numbers (gaps: 2-4, 6-9)
            Test 6: Some lines have non-numeric line numbers
            Test 7: Config file has lines with wrong token count
            Test 8: [scene] tag with no title following it
            Test 9: Extra tokens after config filename in script file

        Example program output see: test/test_0/tmp.txt

Overview:
    The program is organized into six main modules that separate concerns:
        declarations.rs: Defines constants, exit codes, and global configuration (WHINGE_MODE)
        main.rs: Entry point handling command-line parsing and orchestrating the overall flow
        play.rs: Top-level structure managing multiple scene fragments
        scene_fragment.rs: Represents individual scenes with their cast of players
        player.rs: Manages individual character dialogue and line delivery
        script_gen.rs: Utility functions for file I/O operations
        return_wrapper.rs: Custom return type for proper exit code handling

    The program data flow:  Script File -> Play -> SceneFragments -> Players -> Individual Lines
        - main.rs parses command-line arguments and enables WHINGE_MODE if requested
        - Play::prepare() reads the master script file line by line
        - Script lines beginning with [scene] are identified as scene titles
        - Other lines are treated as configuration filenames for scenes
        - Each SceneFragment is created with its title
        - SceneFragment::prepare() reads its configuration file
        - Configuration maps character names to their script files
        - Each Player is instantiated and prepares their lines
        - Players read their script files and parse (line_number, text) pairs
        - Lines are sorted by number to handle out-of-order input
        - Players within each scene are sorted (by first line number)
        - Play::recite() iterates through all scene fragments
        - For each scene, appropriate enter/exit directives are printed
        - SceneFragment::recite() orchestrates dialogue delivery
        - The recite loop repeatedly finds the player with the smallest next line number
        - That player speaks their line via Player::speak()
        - Process continues until all players exhaust their lines

Insights:
    Implementing PartialOrd and Ord for Player to enable sorting makes scene_fragment.rs very clean.
    Rust's sort() method "just worked" once we defined what it means for one player to be less than another.

How to run:
    use cmd to unzip the folder: unzip lab2.zip
    to build the project: cargo build
    Now the program can be run using: target/debug/lab2 <script_filename> [whinge]
    [Note: the script file and part files must be in the root of the directory]

Running Provided Tests:
    IMPORTANT: You need to cd into the directory of test files first. For example to run test_1 cd test/test_0
    Example CMDS:
        ../../target/debug/lab2 partial_hamlet_act_ii_script.txt




