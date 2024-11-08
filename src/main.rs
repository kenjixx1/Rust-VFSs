
use iced::Theme;
use iced::{
    executor,
    widget::{
        button, column, container, horizontal_space, row, text, text_editor,
        vertical_space, PickList,
    },
    Application, Command, Settings,
};
use std::collections::VecDeque;

use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;

#[derive(Debug, Clone)]
enum FileType {
    Dir(String),
    File(String),
}

#[derive(Debug)]
struct FileSystem {
    content: text_editor::Content,
    file_content: text_editor::Content,
    dir: Option<PathBuf>,
    error: Option<Error>,
    show_menu: bool,
    mode: Mode,
    modecount: u32,
    theme: Theme,
    clipboard: Option<PathBuf>,
}
#[derive(Debug, Clone)]
enum Mode {
    Start,
    Opened,
    OnDir,
    ConfirmDel,
    ThemePage,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    LoadFolder(Result<(PathBuf, Vec<FileType>), Error>),
    LoadFileFolder(Result<(PathBuf, Vec<FileType>), Error>),
    LoadFile(Result<(PathBuf, Arc<String>), Error>),
    OpenFile,
    OpenFolder,
    New,
    Show,
    Save,
    Delete,
    ConfirmDelete,
    FileDeleted(Result<PathBuf, Error>),
    FileSaved(Result<PathBuf, Error>),
    CreateFolder,
    CreatedFolder(Result<PathBuf, Error>),
    GoEditPage,
    GoThemePage,
    GoDirPage,
    SelectedTheme(Theme),
    BackFolder,
    Copy,
    Paste,
    Refresh,
    // BackedFolder(Result<(PathBuf, Vec<FileType>), Error>),
}

impl Application for FileSystem {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                file_content: text_editor::Content::new(),
                dir: None,
                error: None,
                show_menu: false,
                mode: Mode::Start,
                modecount: 0,
                theme: Theme::Dark,
                clipboard: None,
            },
            Command::perform(read_directory(default_file()), Message::LoadFolder),
        )
    }

    fn title(&self) -> String {
        format!("File System")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Edit(action) => self.file_content.perform(action),
            Message::LoadFile(result) => {
                self.mode = Mode::Opened;
                if let Ok((path, content)) = result {
                    self.mode = Mode::Opened;
                    self.dir = Some(path.clone());
                    self.file_content = text_editor::Content::with_text(&content);
                    println!("File loaded successfully");
                    return Command::perform(read_file_directory(path), Message::LoadFileFolder);
                } else {
                    self.error = result.err()
                }
            }
            Message::GoDirPage => {
                self.mode = Mode::OnDir;
                let mut file_path = self.dir.clone().unwrap();
                file_path.pop();
                self.dir = Some(file_path);
                self.show_menu = false;
                return Command::none();
            }
            Message::LoadFolder(result) => {
                if self.modecount == 0 {
                    println!("Mode Count{}", self.modecount);
                    self.modecount += 1;
                    return Command::none();
                } else {
                    self.mode = Mode::OnDir;
                    println!("Mode Count{}", self.modecount);
                    let mut text = String::new();
                    if let Ok(folder) = result {
                        self.dir = Some(folder.0);
                        for filename in folder.1 {
                            match filename {
                                FileType::Dir(var) => {
                                    text.push_str(&format!("📁{}\n", var));
                                }
                                FileType::File(var) => {
                                    text.push_str(&format!("📝{}\n", var));
                                }
                            }
                        }

                        self.file_content = text_editor::Content::new();
                        self.content = text_editor::Content::with_text(&text);
                    }
                }
            }
            Message::LoadFileFolder(result) => {
                let mut text = String::new();
                println!("Changing File/Folder");
                if let Ok(folder) = result {
                    for filename in folder.1 {
                        match filename {
                            FileType::Dir(var) => {
                                text.push_str(&format!("📁{}\n", var));
                            }
                            FileType::File(var) => {
                                text.push_str(&format!("📝{}\n", var));
                            }
                        }
                    }
                    self.content = text_editor::Content::with_text(&text);
                }
            }
            Message::BackFolder => {
                let mut file_path = self.dir.clone().unwrap();
                file_path.pop();
                self.dir = Some(file_path);
                return Command::perform(
                    read_directory(self.dir.clone().unwrap()),
                    Message::LoadFolder,
                );
            }
            Message::New => {
                self.mode = Mode::Opened;
                self.file_content = text_editor::Content::new();
                self.show_menu = false;
                let text = self.file_content.text();
                if self.dir.is_none() {
                    self.dir = Some(default_file());
                    return Command::perform(save_file(self.dir.clone(), text), Message::FileSaved);
                }
                if !self.dir.clone().unwrap().is_dir() {
                    let mut file_path = self.dir.clone().unwrap();
                    file_path.pop();
                    self.dir = Some(file_path);
                    return Command::perform(save_file(self.dir.clone(), text), Message::FileSaved);
                } else {
                    return Command::perform(save_file(self.dir.clone(), text), Message::FileSaved);
                }
            }
            Message::Show => {
                self.show_menu = !self.show_menu;
            }
            Message::Save => {
                let text = self.file_content.text();
                self.show_menu = false;
                return Command::perform(save_file(self.dir.clone(), text), Message::FileSaved);
            }
            Message::FileSaved(result) => {
                if let Err(error) = result {
                    self.error = Some(error);
                } else {
                    self.dir = Some(result.unwrap());
                    return Command::perform(
                        read_file_directory(self.dir.clone().unwrap()),
                        Message::LoadFileFolder,
                    );
                }
            }
            Message::Delete => {
                self.mode = Mode::ConfirmDel;
                return Command::none();
            }
            Message::ConfirmDelete => {
                return Command::perform(delete_file(self.dir.clone()), Message::FileDeleted)
            }
            Message::FileDeleted(result) => {
                if let Err(error) = result {
                    self.error = Some(error);
                } else {
                    self.file_content = text_editor::Content::new();
                    self.mode = Mode::OnDir;
                    self.show_menu = false;
                    return Command::perform(
                        read_file_directory(self.dir.clone().unwrap()),
                        Message::LoadFileFolder,
                    );
                }
            }
            Message::CreateFolder => {
                return Command::perform(create_folder(self.dir.clone()), Message::CreatedFolder)
            }
            Message::CreatedFolder(result) => {
                if let Err(error) = result {
                    self.error = Some(error);
                } else {
                    self.dir = Some(result.unwrap());
                    return Command::perform(
                        read_file_directory(self.dir.clone().unwrap()),
                        Message::LoadFileFolder,
                    );
                }
            }
            Message::GoEditPage => {
                self.mode = Mode::Opened;
                return Command::none();
            }
            Message::GoThemePage => {
                self.mode = Mode::ThemePage;
                return Command::none();
            }
            Message::Copy => {
                self.clipboard = self.dir.clone();
                println!("Copied {:?}", self.clipboard);
                return Command::none();
            }
            Message::Paste => {
                if let Some(og_path) = self.clipboard.clone() {
                    if let Some(dest_path) = self.dir.clone() {
                        if og_path.is_dir() {
                            return Command::perform(
                                async move {
                                    copy_foldder(&og_path, &dest_path)
                                        .await
                                        .map(|_| (dest_path.clone(), vec![]))
                                },
                                Message::LoadFileFolder,
                            );
                        } else {
                            return Command::perform(
                                async move {
                                    let og_path_clone = og_path.clone();
                                    copy_file(&og_path_clone, &dest_path)
                                        .await
                                        .map(|_| (dest_path.clone(), vec![]))
                                },
                                Message::LoadFileFolder,
                            );
                        }
                    }
                }
            }
            Message::Refresh => {
                return Command::perform(
                    read_directory(self.dir.clone().unwrap()),
                    Message::LoadFolder,
                )
            }
            Message::SelectedTheme(th) => {
                self.theme = th;
                self.mode = Mode::Opened;
                return Command::none();
            }
            Message::OpenFile => return Command::perform(open_file(), Message::LoadFile),
            Message::OpenFolder => return Command::perform(open_folder(), Message::LoadFolder),
        }
        Command::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message> {
        match self.mode {
            Mode::Start => {
                let open_file_button: button::Button<'_, Message> =
                    button(text("Open File").size(40)).on_press(Message::OpenFile);
                let open_folder_button: button::Button<'_, Message> =
                    button(text("Open Folder").size(40)).on_press(Message::OpenFolder);
                let button_row = row![
                    horizontal_space(),
                    open_file_button,
                    text("    ").size(40),
                    open_folder_button,
                    horizontal_space()
                ]
                .padding(10);
                let text_row = row![
                    horizontal_space(),
                    text("Rust Virtual File System").size(50),
                    horizontal_space()
                ];
                container(column![
                    vertical_space(),
                    text_row,
                    button_row,
                    vertical_space()
                ])
                .padding(50)
                .into()
            }
            Mode::Opened => {
                let txt = text("Files System  ||     ");
                let txt_edit = text_editor(&self.content)
                    .on_action(Message::Edit)
                    .height(1080);

                let pathh: &str = self
                    .dir
                    .as_deref()
                    .and_then(Path::to_str)
                    .unwrap_or("No Folder Selected");
                let go_dir = button("Back").on_press(Message::GoDirPage);
                let copy_button = button("Copy").on_press(Message::Copy);
                let path_text = text(pathh);
                let load_folder_button: button::Button<'_, Message> =
                    button("Select Folder").on_press(Message::OpenFolder);
                let load_file_button: button::Button<'_, Message> =
                    button("Load File").on_press(Message::OpenFile);
                let new_button: button::Button<'_, Message> = button("New").on_press(Message::New);
                let save_button: button::Button<'_, Message> =
                    button("Save").on_press(Message::Save);
                let show_menu = button("Show Options").on_press(Message::Show);
                let space = text("    ");
                let delete_button = button("Delete").on_press(Message::Delete);
                let change_theme = button("Change Theme").on_press(Message::GoThemePage);

                if self.show_menu {
                    let toprow = row![
                        txt,
                        path_text,
                        space.clone(),
                        show_menu,
                        space.clone(),
                        new_button,
                        space.clone(),
                        save_button,
                        space.clone(),
                        delete_button,
                        horizontal_space(),
                        change_theme,
                        space.clone(),
                        copy_button,
                        space.clone(),
                        go_dir,
                        space.clone(),
                        load_folder_button,
                        space.clone(),
                        load_file_button
                    ]
                    .padding(5);
                    let txt_content = text_editor(&self.file_content)
                        .on_action(Message::Edit)
                        .height(1000);

                    let container1 = container(column![txt_edit]).padding(10).width(200);

                    let container2 = container(column![txt_content]).padding(10);

                    column![toprow, row![container1, container2]].into()
                } else {
                    let toprow = row![
                        txt,
                        path_text,
                        space.clone(),
                        show_menu,
                        horizontal_space(),
                        space.clone(),
                        load_folder_button,
                        space.clone(),
                        load_file_button
                    ]
                    .padding(5);
                    let txt_content = text_editor(&self.file_content)
                        .on_action(Message::Edit)
                        .height(1000);

                    let container1 = container(column![txt_edit]).padding(10).width(200);

                    let container2 = container(column![txt_content]).padding(10);

                    column![toprow, row![container1, container2]].into()
                }
            }
            Mode::OnDir => {
                let vspace = text("\n");
                let hspace = text("    ");
                let txt_edit = text_editor(&self.content)
                    .on_action(Message::Edit)
                    .height(1080);
                let pathh: &str = self
                    .dir
                    .as_deref()
                    .and_then(Path::to_str)
                    .unwrap_or("No Folder Selected");
                let txt = text("Files System  || ");
                let path_text = text(pathh);
                let new_file_button: button::Button<'_, Message> =
                    button(text("Create File").size(35)).on_press(Message::New);
                let new_folder_button: button::Button<'_, Message> =
                    button(text("Create Folder").size(35)).on_press(Message::CreateFolder);
                let select_folder_button: button::Button<'_, Message> =
                    button(text("Select Folder").size(35)).on_press(Message::OpenFolder).width(230);
                let select_file_button: button::Button<'_, Message> =
                    button(text("Select File").size(35)).on_press(Message::OpenFile);
                let paste_button = button(text("Paste").size(35).horizontal_alignment(iced::alignment::Horizontal::Center)).on_press(Message::Paste).width(125);
                let copy_button = button(text("Copy").size(35).horizontal_alignment(iced::alignment::Horizontal::Center)).on_press(Message::Copy).width(125);
                let refresh_button = button(text("Refresh").size(35).horizontal_alignment(iced::alignment::Horizontal::Center)).on_press(Message::Refresh).width(125);
                let toprow = row![txt, path_text];
                let exit_button = button(
                    text("Exit Folder")
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .size(35),
                )
                .on_press(Message::BackFolder)
                .width(410);
                let all_button = column![
                    vertical_space(),
                    row![new_file_button, hspace.clone(), new_folder_button],
                    vspace.clone(),
                    row![select_file_button, hspace.clone(), select_folder_button],
                    vspace.clone(),
                    row![exit_button],
                    
                    vertical_space(),
                    row![
                        paste_button,
                        hspace.clone(),
                        copy_button,
                        hspace.clone(),
                        refresh_button
                    ],
                    vspace.clone()
                    
                ];
                let container1 = container(column![txt_edit]).padding(10).width(700);
                let container2 =
                    container(row![horizontal_space(), all_button, horizontal_space()]).padding(10);
                column![toprow, row![container1, container2]].into()
            }

            Mode::ConfirmDel => {
                let confirm_text = text("Are you sure you want to delete this file?").size(30);
                let yes_button: button::Button<'_, Message> =
                    button(text("Yes").size(40)).on_press(Message::ConfirmDelete);
                let no_button: button::Button<'_, Message> =
                    button(text("No").size(40)).on_press(Message::GoEditPage);
                let con_text = row![horizontal_space(), confirm_text, horizontal_space()];
                let button_row = row![
                    horizontal_space(),
                    yes_button,
                    text("    ").size(40),
                    no_button,
                    horizontal_space()
                ]
                .padding(10);

                container(column![
                    vertical_space(),
                    con_text,
                    button_row,
                    vertical_space()
                ])
                .padding(50)
                .into()
            }
            Mode::ThemePage => {
                let picklist =
                    PickList::new(Theme::ALL, Some(self.theme.clone()), Message::SelectedTheme);

                container(row![horizontal_space(),column![vertical_space(),picklist,vertical_space()],horizontal_space()]).padding(50).into()
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }
}

#[derive(Debug, Clone)]
enum Error {
    DClosed,
    IOErr(io::ErrorKind),
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}", env!("CARGO_MANIFEST_DIR")))
}

async fn read_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;

    Ok((path, content))
}

async fn read_directory(path: PathBuf) -> Result<(PathBuf, Vec<FileType>), Error> {
    println!("Reading directory: {:?}", path);
    let mut handle = tokio::fs::read_dir(&path)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;

    let mut fsname = Vec::new();
    while let Some(file) = handle.next_entry().await.unwrap_or(None) {
        let file_name = file.file_name();
        let metadata = file.metadata().await.unwrap();
        if metadata.is_dir() {
            fsname.push(FileType::Dir(file_name.to_string_lossy().to_string()))
        } else {
            fsname.push(FileType::File(file_name.to_string_lossy().to_string()));
        }
    }

    Ok((path, fsname))
}

async fn read_file_directory(path: PathBuf) -> Result<(PathBuf, Vec<FileType>), Error> {
    let mut file_path = path.clone();
    file_path.pop();
    println!("Reading directory: {:?}", file_path);
    let mut handle = tokio::fs::read_dir(&file_path)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;

    let mut fsname = Vec::new();
    while let Some(file) = handle.next_entry().await.unwrap_or(None) {
        let file_name = file.file_name();
        let metadata = file.metadata().await.unwrap();
        if metadata.is_dir() {
            fsname.push(FileType::Dir(file_name.to_string_lossy().to_string()))
        } else {
            fsname.push(FileType::File(file_name.to_string_lossy().to_string()));
        }
    }

    Ok((file_path, fsname))
}

async fn create_folder(path: Option<PathBuf>) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        if path.is_dir() {
            rfd::AsyncFileDialog::new()
                .set_title("Choose Folder Name")
                .save_file()
                .await
                .ok_or(Error::DClosed)?
                .path()
                .to_owned()
        } else {
            path
        }
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choost Folder Name")
            .save_file()
            .await
            .ok_or(Error::DClosed)?
            .path()
            .to_owned()
    };
    println!("Creating Folder");
    tokio::fs::create_dir(&path)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;
    Ok(path)
}

async fn open_folder() -> Result<(PathBuf, Vec<FileType>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Select File")
        .pick_folder()
        .await
        .ok_or(Error::DClosed)?;
    read_directory(handle.path().to_owned()).await
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Select File")
        .pick_file()
        .await
        .ok_or(Error::DClosed)?;
    read_file(handle.path().to_owned()).await
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        if path.is_dir() {
            rfd::AsyncFileDialog::new()
                .set_title("Choose File Name")
                .save_file()
                .await
                .ok_or(Error::DClosed)?
                .path()
                .to_owned()
        } else {
            path
        }
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choost File Name")
            .save_file()
            .await
            .ok_or(Error::DClosed)?
            .path()
            .to_owned()
    };

    tokio::fs::write(&path, &text)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;

    Ok(path)
}

async fn delete_file(path: Option<PathBuf>) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        if path.is_dir() {
            rfd::AsyncFileDialog::new()
                .set_title("Choose File Name")
                .save_file()
                .await
                .ok_or(Error::DClosed)?
                .path()
                .to_owned()
        } else {
            path
        }
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choost File Name")
            .save_file()
            .await
            .ok_or(Error::DClosed)?
            .path()
            .to_owned()
    };

    tokio::fs::remove_file(&path)
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;
        
    let mut newpath=path.clone();
    newpath.pop();
    let newpath = newpath;

    Ok(newpath)
}

async fn copy_file(ogfile: &PathBuf, destination: &PathBuf) -> Result<PathBuf, Error> {
    let mut dest_path = destination.clone();
    dest_path.push(ogfile.file_name().unwrap());
    tokio::fs::copy(ogfile, dest_path.clone())
        .await
        .map_err(|error| error.kind())
        .map_err(Error::IOErr)?;
    Ok(dest_path)
}

async fn copy_foldder(ogfolder: &PathBuf, destination: &PathBuf) -> Result<PathBuf, Error> {
    let mut dest_path = destination.clone();
    dest_path.push(ogfolder.file_name().unwrap());
    fs::create_dir_all(&dest_path)
        .await
        .map_err(|error| Error::IOErr(error.kind()))?;
    let mut stack = VecDeque::new();
    stack.push_back((ogfolder.clone(), dest_path.clone()));

    while let Some((current_source, current_dest)) = stack.pop_front() {
        let mut dir = fs::read_dir(&current_source)
            .await
            .map_err(|error| Error::IOErr(error.kind()))?;
        while let Some(entry) = dir.next_entry().await.unwrap_or(None) {
            let path = entry.path();
            let mut new_dest = current_dest.clone(); // reclone the current destination
            new_dest.push(path.file_name().unwrap()); // push the file name to the destination WORK SOMEHOW LMAFO
            if path.is_dir() {
                fs::create_dir_all(&new_dest)
                    .await
                    .map_err(|error| Error::IOErr(error.kind()))?;
                stack.push_back((path, new_dest));
            } else {
                copy_file(&path, &new_dest).await?;
            }
        }
    }

    Ok(dest_path)
}

fn main() -> iced::Result {
    FileSystem::run(Settings::default())
}
