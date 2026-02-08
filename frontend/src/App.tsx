import { useEffect, useState } from "react";
import { MainWindow } from "./pages/main-window";
import { SetupWizard } from "./components/setup-wizard";
import { commands, type Config } from "./lib/tauri";

function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands
      .getConfig()
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const handleSetupComplete = async () => {
    if (config) {
      const updated = { ...config, first_run_complete: true };
      await commands.updateConfig(updated);
      setConfig(updated);
    }
  };

  if (loading) {
    return (
      <div className="h-screen bg-[#0f0f11] flex items-center justify-center">
        <div className="text-muted-foreground text-sm">Loading...</div>
      </div>
    );
  }

  if (!config?.first_run_complete) {
    return (
      <div className="dark">
        <SetupWizard onComplete={handleSetupComplete} />
      </div>
    );
  }

  return (
    <div className="dark">
      <MainWindow />
    </div>
  );
}

export default App;
