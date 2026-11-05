using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Windows.Forms;
using Microsoft.Win32;

[assembly: AssemblyTitle("清风输入法 AI 划词插件安装程序")]
[assembly: AssemblyDescription("Install the Qingfeng AI selection translation/polish plugin")]
[assembly: AssemblyVersion("0.127.0.2")]
[assembly: AssemblyFileVersion("0.127.0.2")]

namespace QingfengAIPlugin
{
    internal static class Program
    {
        [STAThread]
        private static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new InstallerForm());
        }
    }

    internal sealed class InstallerForm : Form
    {
        private readonly TextBox pathBox;
        private readonly Label statusLabel;
        private readonly Button installButton;
        private readonly Button uninstallButton;
        private readonly Button browseButton;

        public InstallerForm()
        {
            Text = "清风输入法 AI 划词插件";
            ClientSize = new Size(660, 390);
            StartPosition = FormStartPosition.CenterScreen;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            Font = new Font("Microsoft YaHei UI", 9F);

            Label title = new Label();
            title.Text = "清风输入法 AI 划词插件";
            title.Font = new Font("Microsoft YaHei UI", 18F, FontStyle.Bold);
            title.AutoSize = true;
            title.Location = new Point(24, 22);
            Controls.Add(title);

            Label subtitle = new Label();
            subtitle.Text = "在清风输入法里选中文字，按快捷键直接翻译或润色。";
            subtitle.ForeColor = Color.DimGray;
            subtitle.AutoSize = true;
            subtitle.Location = new Point(26, 66);
            Controls.Add(subtitle);

            Label pathLabel = new Label();
            pathLabel.Text = "清风输入法安装目录";
            pathLabel.AutoSize = true;
            pathLabel.Location = new Point(26, 112);
            Controls.Add(pathLabel);

            pathBox = new TextBox();
            pathBox.Location = new Point(26, 136);
            pathBox.Width = 500;
            Controls.Add(pathBox);

            browseButton = new Button();
            browseButton.Text = "浏览...";
            browseButton.Location = new Point(536, 134);
            browseButton.Width = 96;
            browseButton.Height = 26;
            browseButton.Click += delegate { Browse(); };
            Controls.Add(browseButton);

            statusLabel = new Label();
            statusLabel.Text = "正在查找清风输入法...";
            statusLabel.AutoSize = false;
            statusLabel.Location = new Point(26, 184);
            statusLabel.Size = new Size(606, 54);
            statusLabel.ForeColor = Color.DimGray;
            Controls.Add(statusLabel);

            Label note = new Label();
            note.Text = "只替换 wind_input.exe 和 wind_setting.exe。\n不会修改 wind_tsf.dll、词库和用户配置。";
            note.AutoSize = true;
            note.ForeColor = Color.DimGray;
            note.Location = new Point(26, 246);
            Controls.Add(note);

            installButton = new Button();
            installButton.Text = "安装插件";
            installButton.Location = new Point(342, 326);
            installButton.Width = 140;
            installButton.Height = 38;
            installButton.Click += delegate { Install(); };
            Controls.Add(installButton);

            uninstallButton = new Button();
            uninstallButton.Text = "卸载 / 恢复";
            uninstallButton.Location = new Point(492, 326);
            uninstallButton.Width = 140;
            uninstallButton.Height = 38;
            uninstallButton.Click += delegate { Uninstall(); };
            Controls.Add(uninstallButton);

            AcceptButton = installButton;
            Shown += delegate { Detect(); };
        }

        private void Detect()
        {
            string root = FindInstallRoot();
            if (root == null)
            {
                pathBox.Text = @"C:\Program Files\WindInput";
                SetStatus("没有自动找到清风输入法。请先安装官方 0.127.0，或手动选择安装目录。", Color.Firebrick);
                return;
            }
            pathBox.Text = root;
            if (IsValidRoot(root))
            {
                SetStatus("已找到清风输入法：" + root, Color.SeaGreen);
            }
            else
            {
                SetStatus("目录里缺少 wind_input.exe 或 wind_tsf.dll，请确认是清风输入法安装目录。", Color.Firebrick);
            }
        }

        private void Browse()
        {
            using (FolderBrowserDialog dialog = new FolderBrowserDialog())
            {
                dialog.Description = "选择清风输入法安装目录（包含 wind_input.exe 和 wind_tsf.dll）";
                if (Directory.Exists(pathBox.Text))
                {
                    dialog.SelectedPath = pathBox.Text;
                }
                if (dialog.ShowDialog(this) == DialogResult.OK)
                {
                    pathBox.Text = dialog.SelectedPath;
                    if (IsValidRoot(pathBox.Text))
                    {
                        SetStatus("目录有效，可以安装。", Color.SeaGreen);
                    }
                    else
                    {
                        SetStatus("这个目录里没有 wind_input.exe 或 wind_tsf.dll。", Color.Firebrick);
                    }
                }
            }
        }

        private void Install()
        {
            string root = Normalize(pathBox.Text);
            if (!IsValidRoot(root))
            {
                DialogResult answer = MessageBox.Show(
                    this,
                    "没有找到清风输入法基础文件（wind_input.exe / wind_tsf.dll）。\n\n"
                    + "本插件必须先安装官方清风输入法 0.127.0。\n"
                    + "要打开官方下载页吗？",
                    "缺少清风输入法",
                    MessageBoxButtons.YesNo,
                    MessageBoxIcon.Warning);
                if (answer == DialogResult.Yes)
                {
                    OpenUrl("https://windinput.com/download");
                }
                return;
            }

            string version = GetFileVersion(Path.Combine(root, "wind_input.exe"));
            if (version != null && !version.StartsWith("0.127.", StringComparison.Ordinal))
            {
                DialogResult answer = MessageBox.Show(
                    this,
                    "检测到基础版本是 " + version + "，本插件按 0.127.0 构建。\n"
                    + "版本不一致可能导致协议不兼容，仍要继续吗？",
                    "版本不一致",
                    MessageBoxButtons.YesNo,
                    MessageBoxIcon.Warning);
                if (answer != DialogResult.Yes)
                {
                    return;
                }
            }

            DialogResult confirm = MessageBox.Show(
                this,
                "将安装到：\n" + root + "\n\n"
                + "安装前会自动备份原来的 wind_input.exe / wind_setting.exe。继续吗？",
                "确认安装",
                MessageBoxButtons.OKCancel,
                MessageBoxIcon.Question);
            if (confirm != DialogResult.OK)
            {
                return;
            }

            SetBusy(true);
            try
            {
                SetStatus("正在停止清风输入法...", Color.DimGray);
                Application.DoEvents();
                StopThisInstall(root);

                string stamp = DateTime.Now.ToString("yyyyMMdd-HHmmss");
                Backup(Path.Combine(root, "wind_input.exe"), stamp);
                Backup(Path.Combine(root, "wind_setting.exe"), stamp);

                SetStatus("正在写入插件文件...", Color.DimGray);
                Application.DoEvents();
                ExtractResource("wind_input.exe", Path.Combine(root, "wind_input.exe"));
                ExtractResource("wind_setting.exe", Path.Combine(root, "wind_setting.exe"));

                SetStatus("已安装。正在尝试启动清风输入法...", Color.SeaGreen);
                Application.DoEvents();
                LaunchNormal(Path.Combine(root, "wind_input.exe"));
                SetStatus("安装完成。\n请从开始菜单启动清风输入法，打开 设置 → AI 工具 填写 API Key。", Color.SeaGreen);
                MessageBox.Show(
                    this,
                    "安装完成。\n\n"
                    + "接下来：\n"
                    + "1. 打开清风输入法设置；\n"
                    + "2. 进入 AI 工具；\n"
                    + "3. 填写 API Key。\n\n"
                    + "默认快捷键：Ctrl+Alt+T 翻译，Ctrl+Alt+S 润色。",
                    "安装完成",
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                SetStatus("安装失败。", Color.Firebrick);
                MessageBox.Show(this, ex.Message, "安装失败", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            finally
            {
                SetBusy(false);
            }
        }

        private void Uninstall()
        {
            string root = Normalize(pathBox.Text);
            if (!IsValidRoot(root))
            {
                MessageBox.Show(this, "请先选择正确的清风输入法安装目录。", "目录无效", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            DialogResult confirm = MessageBox.Show(
                this,
                "将恢复本插件最近一次安装前的 wind_input.exe / wind_setting.exe。\n\n目录：" + root + "\n\n继续吗？",
                "卸载 / 恢复",
                MessageBoxButtons.OKCancel,
                MessageBoxIcon.Question);
            if (confirm != DialogResult.OK)
            {
                return;
            }

            SetBusy(true);
            try
            {
                bool wasRunning = IsProcessRunning(root);
                SetStatus("正在停止清风输入法...", Color.DimGray);
                Application.DoEvents();
                StopThisInstall(root);

                bool restored = false;
                restored |= RestoreLatest(Path.Combine(root, "wind_input.exe"));
                restored |= RestoreLatest(Path.Combine(root, "wind_setting.exe"));
                if (!restored)
                {
                    MessageBox.Show(
                        this,
                        "没有找到 .bak-ai-* 备份。\n请重新安装官方清风输入法 0.127.0 覆盖。",
                        "没有备份",
                        MessageBoxButtons.OK,
                        MessageBoxIcon.Warning);
                    SetStatus("没有可恢复的备份。", Color.Firebrick);
                    return;
                }

                SetStatus("已恢复官方文件。", Color.SeaGreen);
                if (wasRunning)
                {
                    LaunchNormal(Path.Combine(root, "wind_input.exe"));
                }
                MessageBox.Show(this, "已恢复。", "卸载完成", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                SetStatus("恢复失败。", Color.Firebrick);
                MessageBox.Show(this, ex.Message, "恢复失败", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            finally
            {
                SetBusy(false);
            }
        }

        private void Backup(string target, string stamp)
        {
            if (File.Exists(target))
            {
                File.Copy(target, target + ".bak-ai-" + stamp, true);
            }
        }

        private static bool RestoreLatest(string target)
        {
            string root = Path.GetDirectoryName(target);
            string pattern = Path.GetFileName(target) + ".bak-ai-*";
            string[] backups = Directory.GetFiles(root, pattern);
            if (backups.Length == 0)
            {
                return false;
            }
            Array.Sort(backups, StringComparer.OrdinalIgnoreCase);
            File.Copy(backups[backups.Length - 1], target, true);
            return true;
        }

        private static void ExtractResource(string name, string target)
        {
            Assembly assembly = Assembly.GetExecutingAssembly();
            using (Stream input = assembly.GetManifestResourceStream(name))
            {
                if (input == null)
                {
                    throw new InvalidOperationException("安装包内缺少资源：" + name);
                }
                string temp = Path.Combine(Path.GetTempPath(), "qfai-" + Guid.NewGuid().ToString("N") + "-" + name);
                using (FileStream output = File.Create(temp))
                {
                    input.CopyTo(output);
                }
                File.Copy(temp, target, true);
                try { File.Delete(temp); } catch { }
            }
        }

        private static void StopThisInstall(string root)
        {
            StopProcess("wind_input", root);
            StopProcess("wind_setting", root);
            System.Threading.Thread.Sleep(500);
        }

        private static void StopProcess(string name, string root)
        {
            foreach (Process process in Process.GetProcessesByName(name))
            {
                try
                {
                    string path = process.MainModule.FileName;
                    if (string.Equals(Path.GetDirectoryName(path), root, StringComparison.OrdinalIgnoreCase))
                    {
                        process.Kill();
                        process.WaitForExit(3000);
                    }
                }
                catch { }
            }
        }

        private static bool IsProcessRunning(string root)
        {
            foreach (Process process in Process.GetProcessesByName("wind_input"))
            {
                try
                {
                    if (string.Equals(Path.GetDirectoryName(process.MainModule.FileName), root, StringComparison.OrdinalIgnoreCase))
                    {
                        return true;
                    }
                }
                catch { }
            }
            return false;
        }

        private static void LaunchNormal(string exe)
        {
            try
            {
                // 安装程序带 requireAdministrator。直接 Process.Start 会把输入法
                // 也拉成管理员进程，普通应用无法注入。交给现有 Explorer（普通
                // 完整性级别）启动，避免这个权限继承问题。
                Process.Start(new ProcessStartInfo("explorer.exe", "\"" + exe + "\"")
                {
                    UseShellExecute = true
                });
            }
            catch { }
        }

        private static void OpenUrl(string url)
        {
            try { Process.Start(new ProcessStartInfo(url) { UseShellExecute = true }); } catch { }
        }

        private static string FindInstallRoot()
        {
            string fromRegistry = FindFromRegistry();
            if (fromRegistry != null) return fromRegistry;

            foreach (Process process in Process.GetProcessesByName("wind_input"))
            {
                try
                {
                    string root = Path.GetDirectoryName(process.MainModule.FileName);
                    if (IsValidRoot(root)) return root;
                }
                catch { }
            }

            string[] candidates = new string[]
            {
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "WindInput"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86), "WindInput"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs", "WindInput"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "WindInput")
            };
            foreach (string candidate in candidates)
            {
                if (IsValidRoot(candidate)) return candidate;
            }
            return null;
        }

        private static string FindFromRegistry()
        {
            RegistryHive[] hives = new RegistryHive[] { RegistryHive.LocalMachine, RegistryHive.CurrentUser };
            RegistryView[] views = new RegistryView[] { RegistryView.Registry64, RegistryView.Registry32 };
            string[] paths = new string[]
            {
                @"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                @"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"
            };

            foreach (RegistryHive hive in hives)
            {
                foreach (RegistryView view in views)
                {
                    foreach (string path in paths)
                    {
                        try
                        {
                            using (RegistryKey baseKey = RegistryKey.OpenBaseKey(hive, view))
                            using (RegistryKey uninstall = baseKey.OpenSubKey(path))
                            {
                                if (uninstall == null) continue;
                                foreach (string subName in uninstall.GetSubKeyNames())
                                {
                                    try
                                    {
                                        using (RegistryKey app = uninstall.OpenSubKey(subName))
                                        {
                                            if (app == null) continue;
                                            string displayName = app.GetValue("DisplayName") as string;
                                            if (displayName == null) continue;
                                            if (displayName.IndexOf("清风", StringComparison.Ordinal) < 0
                                                && displayName.IndexOf("WindInput", StringComparison.OrdinalIgnoreCase) < 0
                                                && displayName.IndexOf("Wind Input", StringComparison.OrdinalIgnoreCase) < 0)
                                            {
                                                continue;
                                            }
                                            string location = app.GetValue("InstallLocation") as string;
                                            if (IsValidRoot(location)) return Normalize(location);
                                            string uninstallString = app.GetValue("UninstallString") as string;
                                            string fromUninstall = DirectoryFromCommand(uninstallString);
                                            if (IsValidRoot(fromUninstall)) return Normalize(fromUninstall);
                                        }
                                    }
                                    catch { }
                                }
                            }
                        }
                        catch { }
                    }
                }
            }
            return null;
        }

        private static string DirectoryFromCommand(string command)
        {
            if (string.IsNullOrEmpty(command)) return null;
            command = command.Trim();
            if (command.StartsWith("\"", StringComparison.Ordinal))
            {
                int end = command.IndexOf('"', 1);
                if (end > 1)
                {
                    command = command.Substring(1, end - 1);
                }
            }
            else
            {
                int exe = command.IndexOf(".exe", StringComparison.OrdinalIgnoreCase);
                if (exe > 0)
                {
                    command = command.Substring(0, exe + 4);
                }
            }
            try { return Path.GetDirectoryName(command); }
            catch { return null; }
        }

        private static bool IsValidRoot(string root)
        {
            if (string.IsNullOrEmpty(root)) return false;
            try
            {
                root = Normalize(root);
                return File.Exists(Path.Combine(root, "wind_input.exe"))
                    && File.Exists(Path.Combine(root, "wind_tsf.dll"));
            }
            catch { return false; }
        }

        private static string Normalize(string root)
        {
            if (root == null) return "";
            return root.Trim().TrimEnd('\\');
        }

        private static string GetFileVersion(string path)
        {
            try { return FileVersionInfo.GetVersionInfo(path).FileVersion; }
            catch { return null; }
        }

        private void SetStatus(string text, Color color)
        {
            statusLabel.Text = text;
            statusLabel.ForeColor = color;
            statusLabel.Refresh();
        }

        private void SetBusy(bool busy)
        {
            installButton.Enabled = !busy;
            uninstallButton.Enabled = !busy;
            browseButton.Enabled = !busy;
            pathBox.Enabled = !busy;
            Cursor = busy ? Cursors.WaitCursor : Cursors.Default;
        }
    }
}
