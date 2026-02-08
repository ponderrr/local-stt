interface StepModelsProps {
  selected: string[];
  onSelect: (ids: string[]) => void;
  onNext: () => void;
}

export function StepModels({ selected: _selected, onSelect: _onSelect, onNext: _onNext }: StepModelsProps) {
  return <div>TODO: Model selection</div>;
}
