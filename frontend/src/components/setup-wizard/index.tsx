import { useState } from 'react';
import { StepGpu } from './step-gpu';
import { StepModels } from './step-models';
import { StepDownload } from './step-download';
import { StepComplete } from './step-complete';

interface SetupWizardProps {
  onComplete: () => void;
}

const STEPS = ['gpu', 'models', 'download', 'complete'] as const;
type Step = (typeof STEPS)[number];

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const [step, setStep] = useState<Step>('gpu');
  const [selectedModels, setSelectedModels] = useState<string[]>(['large-v3']);

  const next = () => {
    const idx = STEPS.indexOf(step);
    if (idx < STEPS.length - 1) {
      setStep(STEPS[idx + 1]);
    }
  };

  return (
    <div className="h-screen flex items-center justify-center bg-[#0f0f11] p-6">
      <div className="w-full max-w-lg">
        {/* Progress */}
        <div className="flex items-center justify-center gap-2 mb-8">
          {STEPS.map((s, i) => (
            <div
              key={s}
              className={`h-1 w-12 rounded-full transition-colors ${
                STEPS.indexOf(step) >= i ? 'bg-primary' : 'bg-white/[0.08]'
              }`}
            />
          ))}
        </div>

        {step === 'gpu' && <StepGpu onNext={next} />}
        {step === 'models' && (
          <StepModels selected={selectedModels} onSelect={setSelectedModels} onNext={next} />
        )}
        {step === 'download' && <StepDownload models={selectedModels} onNext={next} />}
        {step === 'complete' && <StepComplete onFinish={onComplete} />}
      </div>
    </div>
  );
}
