interface StepCompleteProps {
  onFinish: () => void;
}

export function StepComplete({ onFinish }: StepCompleteProps) {
  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 text-center space-y-6">
      <div className="text-4xl">Microphone</div>
      <h2 className="text-lg font-semibold text-foreground">You're All Set</h2>
      <p className="text-sm text-muted-foreground">
        Press <span className="font-mono text-foreground">Ctrl+Shift+Space</span> anywhere
        to start dictating. WhisperType will transcribe your speech locally — nothing leaves
        your machine.
      </p>
      <button
        onClick={onFinish}
        className="bg-primary hover:bg-primary/90 text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        Start Using WhisperType
      </button>
    </div>
  );
}
