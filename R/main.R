# Load necessary libraries
library(readr)
library(dplyr)
library(plotly)
library(dash)
library(dashCoreComponents)
library(dashHtmlComponents)
library(stringr)

# Read the CSV file
data <- read_csv('./data/output.csv')

# Extract the numeric part and the unit part
data <- data %>%
  mutate(TimeValue = as.numeric(str_extract(Time, "[\\d\\.]+")),
         TimeUnit = str_extract(Time, "[a-z]+"))

# Convert all times to milliseconds
data <- data %>%
  mutate(TimeMillis = ifelse(TimeUnit == "s", TimeValue * 1000, TimeValue))

# Now you can continue as before, but use 'TimeMillis' instead of 'Time':
grouped_data <- data %>%
  group_by(Model) %>%
  summarise(AverageTimeMillis = mean(TimeMillis, na.rm = TRUE))

plot <- plot_ly(grouped_data, x = ~Model, y = ~AverageTimeMillis, type = 'bar')

app <- Dash$new()

app$layout(
  htmlDiv(
    list(
      dccGraph(
        figure = plot
      )
    )
  )
)

app$run_server(debug=TRUE)