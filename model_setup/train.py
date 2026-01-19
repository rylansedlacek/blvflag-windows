import ftsetup
from transformers import Trainer, TrainingArguments

# going to need to train somewhere other than on my laptop maybe Colab
# ask PC about this when meet next

training_args = TrainingArguments(
    output_dir="~/blvflag/models/finetuned/",
    per_device_train_batch_size=4,
    gradient_accumulation_steps=8,
    warmup_steps=100,
    max_steps=1000,  
    learning_rate=2e-4,
    logging_dir="~/blvflag/trainlogs",
    save_strategy="epoch",
    push_to_hub=False
)

trainer = Trainer(
    model=ftsetup.peft_model,
    args=training_args,
    train_dataset="~/blvflag/models/mbpp.jsonl"
)

trainer.train() # train it 

