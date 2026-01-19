<h1><u>blvdiff (formerly blvflag) - Windows </u></h1>

<h2>About:</h2>

- An on-going individual study project at the <i> University of Mary Washington </i>
- Parts of the Rust structure based from blvrun: https://arxiv.org/pdf/2401.16654
- Designed to <b>improve</b> debugging directly on the command line for blind and low-vision (blv) developers.
- The associated paper for this project can be found here: https://dl.acm.org/doi/10.1145/3663547.3759716

<h2>Features:</h2>

- Rust based shell script.
- Automatic script versioning.

- <b>Three Flags</b>

  - <code>--explain</code> flag: feeds error tracebacks into an LLM powered by <a href="https://www.llama.com/">Meta Llama</a>. Outputs an error explanation in a <i>screen-readable</i> format.
  - <code>--diff</code> flag: Utilizes previously saved script versions to generate a <i>screen-readable</i> display of changes made. Similiar functionality to that of vim-diff, but with automatic versioning.
  - <code>--revert</code> flag: Revert current script version back to most recently saved version. Similiar to that of git's revert feature.
 

<h2>Install:</h2>

Note: This version is for Windows only.

For Mac/Linux support: <a href="https://github.com/rylansedlacek/blvflag"> blvdiff </a>

<h3>Steps:</h3>

1) Install <a href="https://doc.rust-lang.org/cargo/getting-started/installation.html">Cargo</a>

2) In your <b>home</b> directory, on your command line type,
  
- <code>cargo install --git https://github.com/rylansedlacek/blvflag</code>

3) Next, ensure that the tool has installed by typing,

- <code>blvflag help</code>
- You should get back the help screen upon typing this.

4) Next type,
- <code>blvflag setup</code> and enter the auth key.

You are now ready to use the flags described above.

Write a python script and run it with
  
- <code>blvflag yourScriptName.py </code>
- <code>blvflag yourScriptName.py --explain </code>
- <code>blvflag yourScriptName.py --diff </code>
- <code>blvflag yourScriptName.py --revert </code>

Note: to clear history directories,

- <code>blvflag clear </code>

---------------------------------------------------------------
<h2>Winter 2024 to ...</h2>

