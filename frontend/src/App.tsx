import { Component, useEffect, useState, type ReactNode } from "react";
import { MainWindow } from "./pages/main-window";
import { SetupWizard } from "./components/setup-wizard";
import { commands, type Config } from "./lib/tauri";

class ErrorBoundary extends Component<
  { children: ReactNode },
  { hasError: boolean; error: string }
> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: "" };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error: error.message };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen bg-[#0f0f11] flex items-center justify-center p-6">
          <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 text-center max-w-md">
            <h2 className="text-lg font-semibold text-foreground mb-2">
              Something went wrong
            </h2>
            <p className="text-sm text-muted-foreground mb-4">
              {this.state.error}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-md text-sm"
            >
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

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

export default function AppWithErrorBoundary() {
  return (
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  );
}
