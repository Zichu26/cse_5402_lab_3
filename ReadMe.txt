Lab number: CSE 5402 Fall 2025 Lab 3
Zichu Pan p.zichu@wustl.edu
Edgar Palomino e.j.palomino@wustl.edu

Rust Package 1: lab3client

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

    Structs:

        Player Struct:
            Instance Variables:
                name: The name of the character
                lines: The character's lines as tuples of line number and text
                index: The current line number
            Methods:
                new(): The constructor of the struct that initializes a new player with a certain name
                prepare(): A method that reads the lines from a part file, processes the line numbers and sorts the lines by line number
                add_script_line(): A helper method that prepare() calls to process each line in the part file
                speak(): A method that finds out who the next speaking character is and delivers the next line in their script
                next_line(): A helper method that returns the line number of the next line to be spoken

        Scene Fragment Struct:
            Instance Variables:
                title: The name of the scene fragment
                players: The characters in the scene fragment as Player structs
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

        Play Struct:
            Instance Variables:
                fragments: The scene fragments of the play as SceneFragment structs
            Methods:
                new(): The constructor of the struct that initializes a new play
                prepare(): A method sets up the plays scenes and generates the character's scripts
                read_config(): A method that reads the settings from a scene configuration file and processes the configuration tokens
                add_config(): A helper method that read_config() calls to process each setting in the configuration file
                process_config(): A method that reads the scene configuration files listed in the script configuration file to generate the character's scripts
                recite(): A method that arranges the delivery of the character's lines in order by line number

    Return Wrapper Struct:
        Instance Variables:
            code: An unsigned 8-bit integer that stores the exit code value
        The Termination trait implementation handles the casting from the return wrapper to a shell exit code. This is done more specificaly in the report()
        method which checks if the code is a non-zero integer, printing a message with the format "Error: {code}" to stderr, and then regardless of the value
        of the code, returning the ExitCode casted as a shell exit code. Following this design, the return values are wrapped with the return wrapper in the
        following way:
            ReturnWrapper::new(0) for successful execution
            ReturnWrapper::new(BAD_COMMAND_LINE_ERROR) for command-line argument errors
            ReturnWrapper::new(SCRIPT_GENERATION_ERROR) for file I/O or parsing errors

    Design Challenges:

        The main challenge involved having the play deliver its lines adhering to the global line number order but with the players only knowing their own lines.
        This was solved in Lab 2 and the approach involved having the recite() method iterating through the players continuously, finding the player with the
        smallest upcoming line number using the next_line() method and then calling the speak() method for that player to recite the line. This continues until
        all the players have no lines left to speak.

        Another challenge involved identifying and informing of missing line numbers when in "WHINGE" mode. This was also tackled within the recite() method, with the
        approach involving keeping track of the expected line number in a variable and comparing this repetitively with the actual line number that is being recited.
        The conditions can be branched out in the following way:
            If actual_line_number > expected_line_number, then there are missing lines and a warning is printed for each gap
            If actual_line_number == expected_line_number, then there are no mistakes with the line numbers and no warning is printed
            If actual_line_number < expected_line_number, then there is a duplicate line number and a warning is printed for it
        
        The last that was encountered and solved in Lab 2 was having players that appear in multiple consecutive scene fragments. In these cases, the expected behaviour
        was having the players avoid exiting and immediately re-entering, which required to implement some conditioning. This was achieved by having the enter() and
        exit() methods compare the lists of players that were in the last scene with the ones in the new scene, only making the players that are not in the old scene
        exit and then enter. With this impilementation, only the first scene uses the enter_all() method and only the last scene uses the exit_all() method.

        Moving on to the challenges in the present assignment, the main difficulty was keeping the global variables and collections that are used in many parts of the
        code thread_safe, which was especially a concern as the primary extension to the Lab 2 codebase in Lab 3 was the inclusion of threads to add parallelism to the
        processing of the input files. To address this, for variables with Copy types such as WHINGE_MODE and CANCEL_FLAG, atomic variables were used to ensure that any
        changes done to them is fully realized before becoming available again for use and for variables that were collections, such as the vector of Player structs in
        the SceneFragment struct or the vector of SceneFragments in the Play struct, each struct stored in them was wrapped in a Mutex to allow for self parallel mutation,
        which was itself wrapped in an Arc for a similar reason to the choice of the atomic variables.

Rust Package 2: lab3server

    Overview:
        This package is another of the main novelties in Lab 3 and it consists of a multi-threaded server that establishes a TCP connection with 1 or more clients, spawning
        a different thread for each one of these.
    
    Structs:
        Server Struct:
            Instance Variables:
                listener: The TCPListener for the server
                listening_address: The network address where the server is being hosted (in IPv4 format with a port)
            Methods:
                new(): The constructor of the struct that initializes a new server with the TCPListener set to none and the listening_address as an empty string
                is_open(): A helper method that returns true if the server has an active listener and false otherwise
                open(): A method that creates a TCPListener bounded to the network address passed as an argument, also initializing the listener and listening_address instance variables of the struct
                run(): A method that contains the main server loop, accepting the client connections that it receives and spawning a new thread every time that this happens
                is_safe_filename(): A helper method to check that the files requested by the client will not be malicious files such as scripts or attempts to do a directory traversal attack
                handle_connection(): A method that reads the token received from an incoming client connection, shutting down the server if it's "quit" and streaming of the requested file back to the client if it's a valid file in the root directory of the server
        
        Return Wrapper:
            Almost identical to the one in the lab3client Rust package and used for the same purpose.
        
        The server program takes 2 arguments, which are the name of the program and a network address in IPv4 format with a port. Finally, an example command of how
        to run this program can be found below:

            ../../target/debug/lab3testclient 127.0.0.1:1313 test.txt (test.txt is an actual test file in the server's root directory)
        
        Note: When looking for files, the server program will only find files that are in its current directory as file paths including / or \ have been disabled to
        avoid the possibility of directory traversal attacks. For example, if this software is ran from the lab3server directory using the cargo new command, it will
        only be able to identify files that are also in the lab3server and not those that are in any sub-directories.

Rust Package 3: lab3client

    Overview:
        The main purpose of this package was to check that the communication between the client and the server could be established properly. This relatively small
        program consists of only a main.rs with only a main() function and a usage() function. It accepts 3 arguments, namely the name of the program, a network
        address in IPv4 format with a port and a token, which can be "quit" or the name of a remote file in the root directory of the server. If the token passed to
        the program is "quit", the server is closed and if the token is instead the name of a remote file, the contents of the remote file are sent back to the client
        and printed in the client's standard output stream. An example command to use this program is the following:

            ../../target/debug/lab3testclient 127.0.0.1:1313 test.txt (test.txt is an actual test file in the server's root directory)
        
        Note: The server needs to be running for this package to work
    
Testing:

    Since the codebase for Lab 3 was an augmentation of the Lab 2 codebase that retained the same functionality, we found it suitable to run the code through the same
    tests that we used for Lab 2 (with the addition of the provided Macbeth test files in the lab write-up). These tests can be found in the /test directory and it
    should be noted that all of them generate at least 1 warning message (namely the "line 0 missing" warning) for testing. The 16 test cases we designed are expanded
    upon below:

    Good Cases:
        Test 0: Example provided (Shakespeare's Hamlet)
        Test 1: Simple two-scene play with different characters
        Test 11: Three scenes demonstrating player continuity
        Test 12: Lines in file are out of order but should be sorted correctly
        Test 13: One player (Oliver) has no lines
        Test 14: The other example provided (Shakespeare's Macbeth)
        
    Error Cases (will fail with an erorr code):
        Test 2: First scene fragment has no title (Output: "Exit code: 3 (SCRIPT_PARSING_ERROR)")
        Test 3: Script file is completely empty (Output: "Exit code: 3 (SCRIPT_PARSING_ERROR)")
        Test 10: Scene config file is empty (Output: "Exit code: 4 (CONFIG_PARSING_ERROR)")

    Warning Cases (will fail but print errors when in "WHINGE" mode):
        Test 4: Player has duplicate line number 2
        Test 5: Missing line numbers (gaps: 2-4, 6-9)
        Test 6: Some lines have non-numeric line numbers
        Test 7: Config file has lines with wrong token count
        Test 8: [scene] tag with no title following it
        Test 9: Extra tokens after config filename in script file
        
    For an example program output see: test/test_0/tmp.txt

    Also, test 15 is not listed under any category because its functioning depends on which script file it is ran with
    
    How to Run:
        use cmd to unzip the folder: unzip lab3.zip
        to build the project: cargo build
        Now the program can be run using: target/debug/lab3client <script_filename> [whinge]
        [Note: the script file and part files must be in the root of the directory]

    Running Provided Tests:
        IMPORTANT: You need to cd into the directory of test files first. For example to run test_1 cd test/test_0
        Example CMDS: ../../target/debug/lab3client partial_hamlet_act_ii_script.txt

    Bash Scripts for Quick Testing:

        To facilitate testing with the test cases we defined, we've provided 2 bash scripts:
            lab3client/test/test_script.sh
            lab3client/test/test_script_whinge.sh

        These can be ran by moving into the lab2/test directory first and then executing them with ./ following the below usage:
            test_script.sh [Start Number (defaults to 0)] [End Number (defaults to 13)]
            test_script_whinge.sh [Start Number (defaults to 0)] [End Number (defaults to 13)]

        IMPORTANT: It is crucial that these scripts are ran from the lab2/test directory for correct functionality and the range
        defined with the Start and End command-line arguments is inclusive (with the default parameters Start=0 and End=13 running
        the 14 tests we've described previously)
        
    Concurrency Testing:
        To test for the presence of concurrency instead of sequentiality, the best test would involve running 2 or more of the given
        tests simultaneously and noticing that the character lines are printed interwoven instead of going step by step test by test
        (which happens, for example, when running the shell script with default arguments). This can be done executing the cargo new
        command at the same time with the "&" operator for 2 or more of the given tests and providing the paths to the script files
        or it can also be facilitated with the bash scripts, with an example of this for test 0 (Shakespeare's Hamlet) and test 14
        (Shakespeare's Macbeth) below:

            ./test_script.sh 0 0 & ./test_script.sh 14 14
        
    Remote Testing:
        To test the program's ability to handle both local files available to lab3client and remote files available to lab3server,
        there are 2 additional tests that we designed based on the standard Hamlet test case. These are test_16 and test_17, where
        test_16 only references files that are available remotely in the lab3server directory and test_17 references both files that
        are available locally to lab3client and remote files. These are not included in the bash scripts as they are different in
        nature to the other tests but they can be ran similarly to the other tests, by moving into the test_16 or test_17 directory
        and running the tests from there. The example commands for them can be found below:

            ../../target/debug/lab3client partial_hamlet_act_ii_script.txt (from inside the test_16 directory)

            ../../target/debug/lab3client partial_hamlet_act_ii_script.txt (from inside the test_16 directory)

    Concurrency and Remote Testing:
        Finally, we wanted to provide a way to test both the concurrency of the multi-threaded server and its capability of streaming
        back files that are requested by the client through a TCP connection. In essence, this would just involve conducting 2 or more
        remote tests (which will test if the program can handle remote files available to lab3server) but having these multiple tests
        run synchronously (which will test if the threading in both lab3client and lab3server are implemented correctly).
